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

package com.exonum.binding.core.storage.indices;

import com.exonum.binding.core.proxy.Cleaner;
import com.exonum.binding.core.storage.database.TemporaryDb;
import com.exonum.binding.test.RequiresNativeLibrary;
import java.util.Objects;
import java.util.stream.Stream;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;

@RequiresNativeLibrary
abstract class BaseIndexGroupTestable {

  /**
   * A default cleaner for a test case.
   */
  Cleaner cleaner;

  TemporaryDb db;

  @BeforeEach
  public void setUp() {
    db = TemporaryDb.newInstance();
    cleaner = new Cleaner();
  }

  @AfterEach
  public void tearDown() {
    Stream.of(cleaner, db)
        .filter(Objects::nonNull)
        .forEach(BaseIndexGroupTestable::close);
  }

  private static void close(AutoCloseable o) {
    try {
      o.close();
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }
}
