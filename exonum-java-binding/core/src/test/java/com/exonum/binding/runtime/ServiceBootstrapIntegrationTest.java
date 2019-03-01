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

package com.exonum.binding.runtime;

import static org.hamcrest.CoreMatchers.containsString;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.exonum.binding.service.AbstractService;
import com.exonum.binding.service.Node;
import com.exonum.binding.service.Schema;
import com.exonum.binding.service.Service;
import com.exonum.binding.service.TransactionConverter;
import com.exonum.binding.service.adapters.UserServiceAdapter;
import com.exonum.binding.service.adapters.UserTransactionAdapter;
import com.exonum.binding.storage.database.MemoryDb;
import com.exonum.binding.storage.database.View;
import com.exonum.binding.test.Bytes;
import com.google.inject.AbstractModule;
import com.google.inject.Inject;
import io.vertx.ext.web.Router;
import java.util.Collections;
import org.junit.jupiter.api.Test;

class ServiceBootstrapIntegrationTest {

  @Test
  void startService() {
    UserServiceAdapter service = ServiceBootstrap.startService(
        UserModule.class.getCanonicalName(), 0);

    // Check the service and its dependencies work as expected.
    assertThat(service.getId(), equalTo(UserService.ID));
    assertThat(service.getName(), equalTo(UserService.NAME));
    short transactionId = 1;
    byte[] payload = Bytes.bytes(0x00, 0x01);

    UserTransactionAdapter transactionAdapter = service.convertTransaction(
        transactionId, payload);

    assertNotNull(transactionAdapter);
    // Check that once startService returns, the native library is loaded. If it’s not,
    // we’ll get an UnsatisfiedLinkError.
    try (MemoryDb database = MemoryDb.newInstance()) {
      assertNotNull(database);
    }
  }

  @Test
  void startServiceNotModule() {
    String invalidModuleName = Object.class.getCanonicalName();

    IllegalArgumentException thrown = assertThrows(IllegalArgumentException.class,
        () -> ServiceBootstrap.startService(invalidModuleName, 0));
    assertThat(thrown.getLocalizedMessage(),
        containsString("class java.lang.Object is not a sub-class "
            + "of com.google.inject.Module"));
  }
}

class UserModule extends AbstractModule {

  @Override
  protected void configure() {
    bind(Service.class)
        .to(UserService.class);

    bind(TransactionConverter.class)
        .toInstance((m) -> (context) -> System.out.println("Transaction#execute"));
  }
}

class UserService extends AbstractService {

  static final short ID = 1;
  static final String NAME = "user-service";

  @Inject
  UserService(TransactionConverter transactionConverter) {
    super(ID, NAME, transactionConverter);
  }

  @Override
  protected Schema createDataSchema(View view) {
    return Collections::emptyList;
  }

  @Override
  public void createPublicApiHandlers(Node node, Router router) {
    // no-op
  }
}