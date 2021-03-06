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

package com.exonum.binding.fakes.services;

import com.exonum.binding.core.runtime.ServiceArtifactId;
import com.exonum.binding.core.runtime.ServiceInstanceSpec;
import com.exonum.binding.core.runtime.ServiceRuntime;
import com.exonum.binding.core.storage.database.Fork;
import com.exonum.binding.fakes.services.invalidservice.NonInstantiableService;
import com.exonum.binding.fakes.services.invalidservice.NonInstantiableServiceModule;
import com.exonum.binding.fakes.services.service.PutValueTransaction;
import com.exonum.binding.fakes.services.service.SchemaFactory;
import com.exonum.binding.fakes.services.service.TestSchema;
import com.exonum.binding.fakes.services.service.TestService;
import com.exonum.binding.fakes.services.service.TestServiceModule;
import com.exonum.binding.test.runtime.ServiceArtifactBuilder;
import java.io.IOException;
import java.nio.file.Path;

/**
 * A utility class creating various service artifacts.
 */
public final class ServiceArtifacts {

  /**
   * Writes a valid service artifact to the specified location. A valid service artifact
   * can be loaded by the {@link ServiceRuntime} and
   * the service can be instantiated.
   * @param artifactId the artifact id
   * @param artifactLocation a path to write the artifact to
   * @throws IOException if it is unable to write the JAR to the given location
   */
  public static void createValidArtifact(ServiceArtifactId artifactId, Path artifactLocation)
      throws IOException {
    new ServiceArtifactBuilder()
        .setPluginId(artifactId.toString())
        .setPluginVersion(artifactId.getVersion())
        .addClasses(PutValueTransaction.class, SchemaFactory.class, TestSchema.class,
            TestService.class)
        .addExtensionClass(TestServiceModule.class)
        .writeTo(artifactLocation);
  }

  /**
   * Writes a service artifact that cannot be loaded. Such artifact will cause an exception
   * during an attempt
   * to {@linkplain ServiceRuntime#deployArtifact(ServiceArtifactId, String) load} it.
   * @param artifactId the artifact id
   * @param artifactLocation a path to write the artifact to
   * @throws IOException if it is unable to write the JAR to the given location
   */
  public static void createUnloadableArtifact(String artifactId, Path artifactLocation)
      throws IOException {
    new ServiceArtifactBuilder()
        .setPluginId(artifactId)
        .setPluginVersion("Invalid version")
        .addClass(TestService.class)
        .writeTo(artifactLocation);
  }

  /**
   * Writes a service artifact that can be loaded, but with a service that cannot be
   * {@linkplain ServiceRuntime#addService(Fork, ServiceInstanceSpec, byte[])}  instantiated}.
   * @param artifactId the artifact id
   * @param artifactLocation a path to write the artifact to
   * @throws IOException if it is unable to write the JAR to the given location
   */
  public static void createWithUninstantiableService(ServiceArtifactId artifactId,
      Path artifactLocation) throws IOException {
    new ServiceArtifactBuilder()
        .setPluginId(artifactId.toString())
        .setPluginVersion(artifactId.getVersion())
        .addClasses(NonInstantiableService.class)
        .addExtensionClass(NonInstantiableServiceModule.class)
        .writeTo(artifactLocation);
  }

  private ServiceArtifacts() {}
}
