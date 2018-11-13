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

package com.exonum.binding.util;

import static com.google.inject.matcher.Matchers.any;
import static com.google.inject.matcher.Matchers.subclassesOf;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.instanceOf;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.exonum.binding.service.Service;
import com.exonum.binding.service.adapters.UserServiceAdapter;
import com.exonum.binding.service.adapters.ViewFactory;
import com.exonum.binding.service.adapters.ViewProxyFactory;
import com.exonum.binding.testutils.LoggingTestUtils;
import com.exonum.binding.transport.Server;
import com.google.inject.AbstractModule;
import com.google.inject.Guice;
import com.google.inject.Injector;
import org.apache.logging.log4j.test.appender.ListAppender;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class LoggingInterceptorTest {

  private static final String EXCEPTION_MESSAGE = "Some exception";

  private ListAppender appender;

  private UserServiceAdapter serviceAdapter;

  @BeforeEach
  void setUp() {
    appender = LoggingTestUtils.getCapturingLogAppender();

    Injector injector = Guice.createInjector(new TestModule());
    serviceAdapter = injector.getInstance(UserServiceAdapter.class);
  }

  @AfterEach
  void tearDown() {
    appender.clear();
  }

  @Test
  void logInterceptedException() {
    Throwable throwable = assertThrows(Throwable.class, () -> serviceAdapter.getId());
    assertThat(throwable, instanceOf(OutOfMemoryError.class));
    assertThat(throwable.getMessage(), equalTo(EXCEPTION_MESSAGE));
    assertThat(appender.getMessages().size(), equalTo(1));
  }

  static class TestModule extends AbstractModule {

    @Override
    protected void configure() {
      Service service = mock(Service.class);
      when(service.getId()).thenThrow(new OutOfMemoryError(EXCEPTION_MESSAGE));
      bind(Service.class).toInstance(service);
      bind(Server.class).toInstance(mock(Server.class));
      bind(ViewFactory.class).toInstance(ViewProxyFactory.getInstance());

      bindInterceptor(subclassesOf(UserServiceAdapter.class), any(), new LoggingInterceptor());

      bind(UserServiceAdapter.class);
    }
  }
}
