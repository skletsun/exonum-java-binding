/*
 * Copyright 2018 The Exonum Team
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

package com.exonum.binding.fakes.services.service;

import com.exonum.binding.core.storage.database.Fork;

final class TestSchemaFactories {

  static <SchemaT> SchemaFactory<SchemaT> createTestSchemaFactory(Fork expectedView,
                                                                  SchemaT schema) {
    return (actualView) -> {
      if (actualView.equals(expectedView)) {
        return schema;
      }
      throw new AssertionError("Unexpected view: " + actualView + ", expected: " + expectedView);
    };
  }

  private TestSchemaFactories() {}
}
