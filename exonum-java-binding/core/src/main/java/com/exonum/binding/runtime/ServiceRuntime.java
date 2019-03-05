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

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

import com.exonum.binding.service.adapters.UserServiceAdapter;
import com.exonum.binding.transport.Server;
import com.google.inject.Injector;
import com.google.inject.Module;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.net.URI;
import java.net.URISyntaxException;

/**
 * A service runtime. It manages the services required for operation of Exonum services (e.g., a
 * {@link Server}; allows the native code to load and unload artifacts (JAR archives with Exonum
 * services), create and stop services defined in the loaded artifacts.
 *
 * <p>This class is thread-safe and does not support client-side locking.
 */
final class ServiceRuntime {

  private final Injector frameworkInjector;

  /**
   * Creates a new runtime with the given framework injector. Starts the server on instantiation.
   *
   * @param frameworkInjector the injector that has been configured with the Exonum framework
   *     bindings. It serves as a parent for service injectors
   * @param serverPort a port for the web server providing transport to Java services
   */
  ServiceRuntime(Injector frameworkInjector, int serverPort) {
    this.frameworkInjector = checkNotNull(frameworkInjector);

    // Start the server
    checkServerIsSingleton(frameworkInjector);
    Server server = frameworkInjector.getInstance(Server.class);
    server.start(serverPort);
  }

  private void checkServerIsSingleton(Injector frameworkInjector) {
    Server s1 = frameworkInjector.getInstance(Server.class);
    Server s2 = frameworkInjector.getInstance(Server.class);
    checkArgument(s1.equals(s2), "%s is not configured as singleton: s1=%s, s2=%s", Server.class,
        s1, s2);
  }

  // todo: How much artifact verification are we willing to perform?

  /**
   * Loads an artifact from the specified location. The loading involves verification of the
   * artifact (i.e., that it is a valid Exonum service; includes a valid service factory).
   *
   * @param serviceArtifactUri a {@linkplain URI URI} from which to load the service artifact
   *     (e.g., a file or network resource)
   * @return a unique service artifact identifier that must be specified in subsequent operations
   *     with it
   * @throws URISyntaxException if the URI is not valid
   * @throws RuntimeException if it failed to load an artifact; or if the given artifact is
   *     already loaded
   */
  @SuppressWarnings("unused")
  String loadArtifact(String serviceArtifactUri) throws URISyntaxException {
    return "com.acme:any-service:1.0.0";
  }

  /**
   * Creates a new service instance of the given type.
   *
   * @param artifactId a unique identifier of the loaded artifact
   *
   * @return a new service
   * @throws IllegalArgumentException if the artifactId is unknown
   * @throws RuntimeException if it failed to instantiate the service
   */
  UserServiceAdapter createService(@SuppressWarnings("unused") String artifactId) {
    // Temporary code for testing purposes: remove
    Module serviceModule = createUserModule(artifactId.equals("artifactId/not-relevant")
            ? "com.exonum.binding.runtime.TestServiceModule" :
            "com.exonum.binding.fakes.services.service.TestServiceModule");
    Injector serviceInjector = frameworkInjector.createChildInjector(serviceModule);
    return serviceInjector.getInstance(UserServiceAdapter.class);
  }




  /**
   * Creates a user module that configures the bindings of that module.
   *
   * @param moduleName a fully-qualified class name of the user service module
   */
  private static Module createUserModule(String moduleName) {
    try {
      Class<?> moduleClass = Class.forName(moduleName);
      Constructor constructor = moduleClass.getDeclaredConstructor();
      Object moduleObject = constructor.newInstance();
      checkArgument(moduleObject instanceof Module, "%s is not a sub-class of %s",
          moduleClass, Module.class.getCanonicalName());
      return (Module) moduleObject;
    } catch (ClassNotFoundException e) {
      throw new IllegalArgumentException("Module class cannot be found", e);
    } catch (IllegalAccessException e) {
      throw new IllegalArgumentException("Cannot access the no-arg module constructor", e);
    } catch (NoSuchMethodException e) {
      throw new IllegalArgumentException("No no-arg constructor", e);
    } catch (InstantiationException | InvocationTargetException e) {
      throw new RuntimeException(e);
    }
  }

  // TODO: unloadArtifact and stopService, once they can be used/ECR-2275
}
