// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use exonum_merkledb::{
    access::{FromAccess, RawAccess},
    proof_map_index::{
        ProofMapIndexIter, ProofMapIndexKeys, ProofMapIndexValues, PROOF_MAP_KEY_SIZE,
    },
    Fork, IndexAddress, ObjectHash, ProofMapIndex, RawProofMapIndex, Snapshot,
};
use exonum_proto::ProtobufConvert;
use handle::{self, Handle};
use jni::{
    objects::{GlobalRef, JClass, JMethodID, JObject, JString},
    sys::{jboolean, jbyteArray, jobject, jobjectArray, JNI_FALSE},
    JNIEnv,
};
use protobuf::Message;
use storage::{
    db::{Key, Value, View, ViewRef},
    PairIter,
};
use utils;
use JniResult;

use std::{panic, ptr};

type RawKey = [u8; 32];

enum Index<T>
where
    T: RawAccess,
{
    Raw(RawProofMapIndex<T, RawKey, Value>),
    Hashed(ProofMapIndex<T, Key, Value>),
}

const JAVA_ENTRY_FQN: &str = "com/exonum/binding/core/storage/indices/MapEntryInternal";

enum IndexType {
    SnapshotIndex(Index<&'static dyn Snapshot>),
    ForkIndex(Index<&'static Fork>),
}

enum Iter<'a> {
    Raw(PairIter<ProofMapIndexIter<'a, RawKey, Value>>),
    Hashed(PairIter<ProofMapIndexIter<'a, Key, Value>>),
}

enum KeysIter<'a> {
    Raw(ProofMapIndexKeys<'a, RawKey>),
    Hashed(ProofMapIndexKeys<'a, Key>),
}

// For easy conversion to RawKey.
trait ToRawKey {
    fn to_raw(&self) -> RawKey;
}

impl ToRawKey for Key {
    fn to_raw(&self) -> RawKey {
        assert_eq!(
            self.len(),
            PROOF_MAP_KEY_SIZE,
            "Key size should be 256 bits"
        );
        let mut result: RawKey = [0; 32];
        result.copy_from_slice(self.as_slice());
        result
    }
}

/// Returns a pointer to the created `ProofMapIndex` object.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreate(
    env: JNIEnv,
    _: JClass,
    name: JString,
    view_handle: Handle,
    key_hashing: jboolean,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let name = utils::convert_to_string(&env, name)?;
        let key_is_hashed = key_hashing != JNI_FALSE;
        Ok(handle::to_handle(
            match handle::cast_handle::<View>(view_handle).get() {
                ViewRef::Snapshot(snapshot) => {
                    if key_is_hashed {
                        IndexType::SnapshotIndex(Index::Hashed(
                            ProofMapIndex::from_access(snapshot, name.into()).unwrap(),
                        ))
                    } else {
                        IndexType::SnapshotIndex(Index::Raw(
                            RawProofMapIndex::from_access(snapshot, name.into()).unwrap(),
                        ))
                    }
                }
                ViewRef::Fork(fork) => {
                    if key_is_hashed {
                        IndexType::ForkIndex(Index::Hashed(
                            ProofMapIndex::from_access(fork, name.into()).unwrap(),
                        ))
                    } else {
                        IndexType::ForkIndex(Index::Raw(
                            RawProofMapIndex::from_access(fork, name.into()).unwrap(),
                        ))
                    }
                }
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the created `ProofMapIndex` instance in an index family (= group).
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreateInGroup(
    env: JNIEnv,
    _: JClass,
    group_name: JString,
    map_id: jbyteArray,
    view_handle: Handle,
    key_hashing: jboolean,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let group_name = utils::convert_to_string(&env, group_name)?;
        let map_id = env.convert_byte_array(map_id)?;
        let address = IndexAddress::with_root(group_name).append_bytes(&map_id);
        let key_is_hashed = key_hashing != JNI_FALSE;
        Ok(handle::to_handle(
            match handle::cast_handle::<View>(view_handle).get() {
                ViewRef::Snapshot(snapshot) => {
                    if key_is_hashed {
                        IndexType::SnapshotIndex(Index::Hashed(
                            ProofMapIndex::from_access(snapshot, address).unwrap(),
                        ))
                    } else {
                        IndexType::SnapshotIndex(Index::Raw(
                            RawProofMapIndex::from_access(snapshot, address).unwrap(),
                        ))
                    }
                }
                ViewRef::Fork(fork) => {
                    if key_is_hashed {
                        IndexType::ForkIndex(Index::Hashed(
                            ProofMapIndex::from_access(fork, address).unwrap(),
                        ))
                    } else {
                        IndexType::ForkIndex(Index::Raw(
                            RawProofMapIndex::from_access(fork, address).unwrap(),
                        ))
                    }
                }
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Destroys the underlying `ProofMapIndex` object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeFree(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
) {
    handle::drop_handle::<IndexType>(&env, map_handle);
}

/// Returns the object hash of the proof map or default hash value if it is empty.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeGetIndexHash(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let hash = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => map.object_hash(),
                Index::Hashed(map) => map.object_hash(),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => map.object_hash(),
                Index::Hashed(map) => map.object_hash(),
            },
        };
        utils::convert_hash(&env, &hash)
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Returns value identified by the `key`. Null pointer is returned if value is not found.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeGet(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        let val = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => map.get(&key.to_raw()),
                Index::Hashed(map) => map.get(&key),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => map.get(&key.to_raw()),
                Index::Hashed(map) => map.get(&key),
            },
        };
        match val {
            Some(val) => env.byte_array_from_slice(&val),
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Returns `true` if the map contains a value for the specified key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeContainsKey(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> jboolean {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => map.contains(&key.to_raw()),
                Index::Hashed(map) => map.contains(&key),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => map.contains(&key.to_raw()),
                Index::Hashed(map) => map.contains(&key),
            },
        } as jboolean)
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns proof that is serialized in protobuf.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeGetProof(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        let proof_proto = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(ref map) => map.get_proof(key.to_raw()).to_pb(),
                Index::Hashed(ref map) => map.get_proof(key).to_pb(),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(ref map) => map.get_proof(key.to_raw()).to_pb(),
                Index::Hashed(ref map) => map.get_proof(key).to_pb(),
            },
        };

        env.byte_array_from_slice(&proof_proto.write_to_bytes().unwrap())
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Returns multiproof that is serialized in protobuf.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeGetMultiProof(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    keys: jobjectArray,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let keys = convert_to_keys(&env, keys)?;
        let proof_proto = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(ref map) => map.get_multiproof(convert_keys(keys)).to_pb(),
                Index::Hashed(ref map) => map.get_multiproof(keys).to_pb(),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(ref map) => map.get_multiproof(convert_keys(keys)).to_pb(),
                Index::Hashed(ref map) => map.get_multiproof(keys).to_pb(),
            },
        };

        env.byte_array_from_slice(&proof_proto.write_to_bytes().unwrap())
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Returns the pointer to the iterator over a map keys and values.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreateEntriesIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let iter_handle = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(ref map) => {
                    handle::to_handle(Iter::Raw(PairIter::new(&env, map.iter(), JAVA_ENTRY_FQN)?))
                }
                Index::Hashed(ref map) => handle::to_handle(Iter::Hashed(PairIter::new(
                    &env,
                    map.iter(),
                    JAVA_ENTRY_FQN,
                )?)),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(ref map) => {
                    handle::to_handle(Iter::Raw(PairIter::new(&env, map.iter(), JAVA_ENTRY_FQN)?))
                }
                Index::Hashed(ref map) => handle::to_handle(Iter::Hashed(PairIter::new(
                    &env,
                    map.iter(),
                    JAVA_ENTRY_FQN,
                )?)),
            },
        };
        Ok(iter_handle)
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over map keys.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreateKeysIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        Ok(match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(KeysIter::Raw(map.keys())),
                Index::Hashed(map) => handle::to_handle(KeysIter::Hashed(map.keys())),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(KeysIter::Raw(map.keys())),
                Index::Hashed(map) => handle::to_handle(KeysIter::Hashed(map.keys())),
            },
        })
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over map values.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreateValuesIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        Ok(match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(map.values()),
                Index::Hashed(map) => handle::to_handle(map.values()),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(map.values()),
                Index::Hashed(map) => handle::to_handle(map.values()),
            },
        })
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over a map keys and values starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeCreateIterFrom(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        let iter_handle = match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(Iter::Raw(PairIter::new(
                    &env,
                    map.iter_from(&key.to_raw()),
                    JAVA_ENTRY_FQN,
                )?)),
                Index::Hashed(map) => handle::to_handle(Iter::Hashed(PairIter::new(
                    &env,
                    map.iter_from(&key),
                    JAVA_ENTRY_FQN,
                )?)),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(Iter::Raw(PairIter::new(
                    &env,
                    map.iter_from(&key.to_raw()),
                    JAVA_ENTRY_FQN,
                )?)),
                Index::Hashed(map) => handle::to_handle(Iter::Hashed(PairIter::new(
                    &env,
                    map.iter_from(&key),
                    JAVA_ENTRY_FQN,
                )?)),
            },
        };
        Ok(iter_handle)
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over map keys starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeKeysFrom(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(KeysIter::Raw(map.keys_from(&key.to_raw()))),
                Index::Hashed(map) => handle::to_handle(KeysIter::Hashed(map.keys_from(&key))),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(KeysIter::Raw(map.keys_from(&key.to_raw()))),
                Index::Hashed(map) => handle::to_handle(KeysIter::Hashed(map.keys_from(&key))),
            },
        })
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over map values starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeValuesFrom(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(match *handle::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(map.values_from(&key.to_raw())),
                Index::Hashed(map) => handle::to_handle(map.values_from(&key)),
            },
            IndexType::ForkIndex(ref index) => match index {
                Index::Raw(map) => handle::to_handle(map.values_from(&key.to_raw())),
                Index::Hashed(map) => handle::to_handle(map.values_from(&key)),
            },
        })
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Sets `value` identified by the `key` into the index.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativePut(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
    value: jbyteArray,
) {
    let res = panic::catch_unwind(|| match *handle::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut index) => {
            let key = env.convert_byte_array(key)?;
            let value = env.convert_byte_array(value)?;
            match index {
                Index::Raw(map) => map.put(&key.to_raw(), value),
                Index::Hashed(map) => map.put(&key, value),
            }
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Removes value identified by the `key` from the index.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeRemove(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) {
    let res = panic::catch_unwind(|| match *handle::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut index) => {
            let key = env.convert_byte_array(key)?;
            match index {
                Index::Raw(map) => map.remove(&key.to_raw()),
                Index::Hashed(map) => map.remove(&key),
            }
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Removes all entries of the map.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeClear(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) {
    let res = panic::catch_unwind(|| match *handle::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut index) => {
            match index {
                Index::Raw(map) => map.clear(),
                Index::Hashed(map) => map.clear(),
            }
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the next value from the iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeEntriesIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jobject {
    let res = panic::catch_unwind(|| {
        let iterWrapper = handle::cast_handle::<Iter>(iter_handle);

        let result = match iterWrapper {
            Iter::Raw(ref mut wrapper) => wrapper.iter.next().map(|(arr, val)| {
                create_element(
                    &env,
                    &arr[..],
                    &val,
                    &wrapper.element_class,
                    wrapper.constructor_id,
                )
            }),
            Iter::Hashed(ref mut wrapper) => wrapper.iter.next().map(|(key, val)| {
                create_element(
                    &env,
                    key.as_slice(),
                    &val,
                    &wrapper.element_class,
                    wrapper.constructor_id,
                )
            }),
        };

        result.or(Some(Ok(ptr::null_mut()))).unwrap()
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `ProofMapIndex` iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeEntriesIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    handle::drop_handle::<Iter>(&env, iter_handle);
}

/// Returns the next value from the keys-iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeKeysIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let array = match *handle::cast_handle::<KeysIter>(iter_handle) {
            KeysIter::Raw(ref mut iter) => iter.next().map(|val| env.byte_array_from_slice(&val)),
            KeysIter::Hashed(ref mut iter) => {
                iter.next().map(|val| env.byte_array_from_slice(&val))
            }
        };
        array.or(Some(Ok(ptr::null_mut()))).unwrap()
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `ProofMapIndex` keys-iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeKeysIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    handle::drop_handle::<KeysIter>(&env, iter_handle);
}

/// Return next value from the values-iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeValuesIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let iter = handle::cast_handle::<ProofMapIndexValues<Value>>(iter_handle);
        match iter.next() {
            Some(val) => env.byte_array_from_slice(&val),
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `ProofMapIndex` values-iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_core_storage_indices_ProofMapIndexProxy_nativeValuesIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    handle::drop_handle::<ProofMapIndexValues<Value>>(&env, iter_handle);
}

// Converts array of Java bytes arrays to the vector of keys.
fn convert_to_keys(env: &JNIEnv, array: jobjectArray) -> JniResult<Vec<Key>> {
    let num_elements = env.get_array_length(array)?;
    let mut keys = vec![];
    for i in 0..num_elements {
        let byte_array: jbyteArray = env.get_object_array_element(array, i)?.into_inner();
        let key = env.convert_byte_array(byte_array)?;
        keys.push(key);
    }
    Ok(keys)
}

// Converts vector of Keys to Vector of RawKeys.
fn convert_keys(keys: Vec<Key>) -> Vec<RawKey> {
    keys.into_iter().map(|key| key.to_raw()).collect()
}

// Creates element for PairIter.
fn create_element(
    env: &JNIEnv,
    key: &[u8],
    value: &[u8],
    class: &GlobalRef,
    constructor: JMethodID,
) -> JniResult<jobject> {
    let key: JObject = env.byte_array_from_slice(key)?.into();
    let value: JObject = env.byte_array_from_slice(value)?.into();
    Ok(env
        .new_object_unchecked(class, constructor, &[key.into(), value.into()])?
        .into_inner())
}
