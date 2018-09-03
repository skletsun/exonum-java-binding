package com.exonum.binding.storage.proofs.map.flat;

import static com.exonum.binding.hash.Funnels.hashCodeFunnel;
import static com.exonum.binding.storage.proofs.DbKeyFunnel.dbKeyFunnel;
import static java.util.stream.Collectors.toList;

import com.exonum.binding.hash.HashCode;
import com.exonum.binding.hash.HashFunction;
import com.exonum.binding.hash.Hashing;
import com.exonum.binding.storage.proofs.map.DbKey;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.Deque;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

/**
 * An unchecked flat map proof, which does not include any intermediate nodes.
 */
public class UncheckedFlatMapProof implements UncheckedMapProof {

  private static final HashFunction HASH_FUNCTION = Hashing.defaultHashFunction();

  private final List<MapProofEntry> proofList = new ArrayList<>();

  private final List<MapProofEntry> proof;

  private final List<MapEntry> leaves;

  private final List<byte[]> absentLeaves;

  UncheckedFlatMapProof(
      List<MapProofEntry> proof,
      List<MapEntry> leaves,
      List<byte[]> absentLeaves) {
    this.proof = proof;
    this.leaves = leaves;
    this.absentLeaves = absentLeaves;
  }

  @SuppressWarnings("unused") // Native API
  static UncheckedFlatMapProof fromUnsorted(
      MapProofEntry[] proofList,
      MapEntry[] leaves,
      byte[] absentLeaves) {
    List<MapProofEntry> proof = Arrays.asList(proofList);
    List<MapEntry> leavesList = Arrays.asList(leaves);
    List<byte[]> absentLeavesList = Arrays.asList(absentLeaves);
    return new UncheckedFlatMapProof(proof, leavesList, absentLeavesList);
  }

  @Override
  public CheckedMapProof check() {
    if (prefixesIncluded()) {
      return CheckedFlatMapProof.invalid(ProofStatus.INVALID_STRUCTURE);
    }
    mergeLeavesWithBranches();
    ProofStatus orderCheckResult = orderCheck();
    if (orderCheckResult != ProofStatus.CORRECT) {
      return CheckedFlatMapProof.invalid(orderCheckResult);
    }
    if (proofList.isEmpty()) {
      return CheckedFlatMapProof.correct(
          getEmptyProofListHash(), Collections.emptyList(), absentLeaves);
    } else if (proofList.size() == 1) {
      // Proof is correct if the only node is a found leaf
      if (leaves.size() == 1) {
          return CheckedFlatMapProof.correct(proofList.get(0).getHash(), leaves, absentLeaves);
      } else {
        return CheckedFlatMapProof.invalid(ProofStatus.NON_TERMINAL_NODE);
      }
    } else {
      Deque<MapProofEntry> contour = new ArrayDeque<>();
      MapProofEntry first = proofList.get(0);
      MapProofEntry second = proofList.get(1);
      DbKey lastPrefix = first.getDbKey().commonPrefix(second.getDbKey());
      contour.push(first);
      contour.push(second);
      for (int i = 2; i < proofList.size(); i++) {
        MapProofEntry currentEntry = proofList.get(i);
        DbKey newPrefix = contour.peek().getDbKey().commonPrefix(currentEntry.getDbKey());
        while (contour.size() > 1
            && newPrefix.getNumSignificantBits() < lastPrefix.getNumSignificantBits()) {
          lastPrefix = fold(contour, lastPrefix).orElse(lastPrefix);
        }
        contour.push(currentEntry);
        lastPrefix = newPrefix;
      }
      while (contour.size() > 1) {
        lastPrefix = fold(contour, lastPrefix).orElse(lastPrefix);
      }
      return CheckedFlatMapProof.correct(contour.peek().getHash(), leaves, absentLeaves);
    }
  }

  /**
   * Compute hashes of leaf entries and merge them into list of branches.
   */
  private void mergeLeavesWithBranches() {
    proofList.addAll(proof);
    List<MapProofEntry> leafEntries =
        leaves
            .stream()
            .map(l -> new MapProofEntry(DbKey.newLeafKey(l.getKey()), getSingleLeafHash(l)))
            .collect(toList());
    proofList.addAll(leafEntries);
    proofList.sort(Comparator.comparing(MapProofEntry::getDbKey));
  }

  /**
   * Check that all entries are in the valid order.
   * The following algorithm is used:
   * Try to find a first bit index at which this key is greater than the other key (i.e., a bit of
   * this key is 1 and the corresponding bit of the other key is 0), and vice versa. The smaller of
   * these indexes indicates the greater key.
   * If there is no such bit, then lengths of these keys are compared and the key with greater
   * length is considered a greater key.
   * Every following key should be greater than the previous.
   * @return {@code ProofStatus.CORRECT} if every following key is greater than the previous
   *         {@code ProofStatus.INVALID_ORDER} if any following key key is lesser than the previous
   *         {@code ProofStatus.DUPLICATE_PATH} if there are two equal keys
   */
  private ProofStatus orderCheck() {
    for (int i = 1; i < proofList.size(); i++) {
      DbKey key = proofList.get(i - 1).getDbKey();
      DbKey nextKey = proofList.get(i).getDbKey();
      int comparisonResult = nextKey.compareTo(key);
      if (comparisonResult < 0) {
        return ProofStatus.INVALID_ORDER;
      } else if (comparisonResult == 0) {
        return ProofStatus.DUPLICATE_PATH;
      }
    }
    return ProofStatus.CORRECT;
  }

  /*
   * Check if any entry has a prefix among the paths in the proof entries. Both found and absent
   * keys are checked.
   */
  private boolean prefixesIncluded() {
    // TODO: entries are sorted, so it's possible to use binary search here
    List<DbKey> requestedKeys =
        Stream.concat(
                leaves.stream().map(l -> DbKey.newLeafKey(l.getKey())),
                absentLeaves.stream().map(DbKey::newLeafKey))
            .collect(toList());
    return requestedKeys
        .stream()
        .anyMatch(leafEntryKey -> proof.stream()
            .map(MapProofEntry::getDbKey)
            .anyMatch(proofEntryKey ->
                leafEntryKey.commonPrefix(proofEntryKey).equals(proofEntryKey))
        );
  }

  /**
   * Folds two last entries in a contour and replaces them with the folded entry.
   * Returns an updated common prefix between two last entries in the contour.
   */
  private Optional<DbKey> fold(Deque<MapProofEntry> contour, DbKey lastPrefix) {
    MapProofEntry lastEntry = contour.pop();
    MapProofEntry penultimateEntry = contour.pop();
    MapProofEntry newEntry =
        new MapProofEntry(lastPrefix, computeBranchHash(penultimateEntry, lastEntry));
    Optional<DbKey> commonPrefix;
    if (!contour.isEmpty()) {
      MapProofEntry previousEntry = contour.peek();
      commonPrefix = Optional.of(previousEntry.getDbKey().commonPrefix(lastPrefix));
    } else {
      commonPrefix = Optional.empty();
    }

    contour.push(newEntry);
    return commonPrefix;
  }

  private static HashCode computeBranchHash(MapProofEntry leftChild, MapProofEntry rightChild) {
    return HASH_FUNCTION
        .newHasher()
        .putObject(leftChild.getHash(), hashCodeFunnel())
        .putObject(rightChild.getHash(), hashCodeFunnel())
        .putObject(leftChild.getDbKey(), dbKeyFunnel())
        .putObject(rightChild.getDbKey(), dbKeyFunnel())
        .hash();
  }

  private static HashCode getEmptyProofListHash() {
    return HashCode.fromBytes(new byte[Hashing.DEFAULT_HASH_SIZE_BYTES]);
  }

  private static HashCode getSingleLeafHash(MapEntry entry) {
    return HASH_FUNCTION.newHasher()
        .putObject(DbKey.newLeafKey(entry.getKey()), dbKeyFunnel())
        .putObject(HASH_FUNCTION.hashBytes(entry.getValue()), hashCodeFunnel())
        .hash();
  }
}
