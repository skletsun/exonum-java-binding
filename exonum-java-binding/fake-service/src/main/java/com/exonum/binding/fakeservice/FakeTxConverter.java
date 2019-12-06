/*
 * Copyright 2019 The Exonum Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.exonum.binding.fakeservice;

import com.exonum.binding.core.service.TransactionConverter;
import com.exonum.binding.core.transaction.Transaction;
import com.exonum.binding.fakeservice.Transactions.PutTransactionArgs;
import com.google.protobuf.InvalidProtocolBufferException;

class FakeTxConverter implements TransactionConverter {

  @Override
  public Transaction toTransaction(int txId, byte[] arguments) {
    try {
      if (txId == PutTransaction.ID) {
        PutTransactionArgs args = PutTransactionArgs.parseFrom(arguments);
        return new PutTransaction(args.getKey(), args.getValue());
      }
      throw new IllegalArgumentException("Unknown transaction: " + txId);
    } catch (InvalidProtocolBufferException e) {
      throw new IllegalArgumentException(e);
    }
  }
}
