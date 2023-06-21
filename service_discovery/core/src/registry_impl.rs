// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Module containing gRPC service implementation based on [`proto_servicediscovery::chariott_registry::v1`].
//!
//! Provides a gRPC endpoint for external services to interact with to register and discover
//! services. Note: Identifiers in all Registry operations are case-sensitive.
//!
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use proto_servicediscovery::chariott_registry::v1::registry_server::Registry;
use proto_servicediscovery::chariott_registry::v1::{
    DiscoverByNamespaceRequest, DiscoverByNamespaceResponse, DiscoverServiceRequest,
    DiscoverServiceResponse, InspectRequest, InspectResponse, RegisterRequest, RegisterResponse,
    RegistrationStatus, ServiceMetadata, UnregisterRequest, UnregisterResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Base structure for the registry gRPC service
#[derive(Clone, Debug)]
pub struct RegistryImpl {
    pub registry_map: Arc<RwLock<HashMap<ServiceIdentifiers, ServiceMetadata>>>,
}

#[tonic::async_trait]
impl Registry for RegistryImpl {
    /// Registers a service by adding it to the registry.
    ///
    /// This function registers a service based on a [`RegisterRequest`]. Returns a
    /// [`RegisterResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - A [`RegisterRequest`] wrapped by a [`tonic::Request`].
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request_inner = request.into_inner();
        let service_to_register =
            request_inner.service.ok_or_else(|| Status::invalid_argument("Service is required"))?;
        let service_identifiers = ServiceIdentifiers {
            namespace: service_to_register.namespace.clone(),
            name: service_to_register.name.clone(),
            version: service_to_register.version.clone(),
        };
        info!("Received a register request for: {:?}", service_identifiers);

        let registration_status;
        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.write();
            match lock.get(&service_identifiers) {
                Some(_) => {
                    lock.insert(service_identifiers.clone(), service_to_register.clone());
                    registration_status = RegistrationStatus::Updated;
                    info!("Updated the service entry in Chariott service registry; overwrote previous entry: {:?}", service_to_register);
                }
                None => {
                    lock.insert(service_identifiers.clone(), service_to_register.clone());
                    registration_status = RegistrationStatus::NewlyRegistered;
                    info!(
                        "Registered new service in Chariott service registry: {:?}",
                        service_to_register
                    );
                }
            };
        }
        let register_response =
            RegisterResponse { registration_status: registration_status as i32 };
        Ok(Response::new(register_response))
    }

    /// Unregisters a service by removing it from the registry.
    ///
    /// This function registers a service based on a [`UnregisterRequest`]. Returns a
    /// [`UnregisterResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - A [`UnregisterRequest`] wrapped by a [`tonic::Request`].
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        let request_inner = request.into_inner();
        let service_identifiers = ServiceIdentifiers {
            namespace: request_inner.namespace,
            name: request_inner.name,
            version: request_inner.version,
        };
        info!("Received an unregister request for: {:?}", service_identifiers);

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.write();
            match lock.remove(&service_identifiers) {
                Some(removed_service) => {
                    info!(
                        "Successfully removed service entry in Chariott service registry: {:?}",
                        removed_service
                    );
                }
                None => {
                    let not_found_message = format!(
                        "Unable to remove service from registry: {:?}",
                        service_identifiers
                    );
                    warn!(not_found_message); //namespace: {0}, name: {1}, version: {2}")
                    return Err(Status::not_found(not_found_message));
                }
            };
        }
        Ok(Response::new(UnregisterResponse {}))
    }

    /// Discovers a list of services based on the namespace, or logical grouping of services.
    ///
    /// This function discovers a list of services based on a [`DiscoverByNamespaceRequest`]. Returns a
    /// [`DiscoverByNamespaceResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - A [`DiscoverByNamespaceRequest`] wrapped by a [`tonic::Request`].
    async fn discover_by_namespace(
        &self,
        request: Request<DiscoverByNamespaceRequest>,
    ) -> Result<Response<DiscoverByNamespaceResponse>, Status> {
        let namespace = request.into_inner().namespace;
        let mut service_list: Vec<ServiceMetadata> = Vec::new();

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.read();
            for (service_identifier, service_metadata) in lock.iter() {
                if service_identifier.namespace == namespace {
                    service_list.push(service_metadata.clone());
                }
            }
        }
        if service_list.is_empty() {
            Err(Status::not_found(format!("No registrations found for namespace {namespace}")))
        } else {
            let discover_by_namespace_response =
                DiscoverByNamespaceResponse { services: service_list };
            Ok(Response::new(discover_by_namespace_response))
        }
    }

    /// Discovers a single service based on its "fully qualified name", consisting of the namespace,
    /// name, and version of the service.
    ///
    /// This function discovers a service based on a [`DiscoverServiceRequest`]. Returns a
    /// [`DiscoverServiceResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - A [`DiscoverServiceRequest`] wrapped by a [`tonic::Request`].
    async fn discover_service(
        &self,
        request: Request<DiscoverServiceRequest>,
    ) -> Result<Response<DiscoverServiceResponse>, Status> {
        let request_inner = request.into_inner();

        let service_identifiers = ServiceIdentifiers {
            namespace: request_inner.namespace.clone(),
            name: request_inner.name.clone(),
            version: request_inner.version,
        };

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.read();
            match lock.get(&service_identifiers) {
                Some(service) => {
                    info!("Read service in DiscoverService {:?}", service);
                    let discover_service_response =
                        DiscoverServiceResponse { service: Some(service.clone()) };
                    Ok(Response::new(discover_service_response))
                }
                None => {
                    let not_found_message = format!(
                        "No service found for namespace: {0}, name: {1}, version: {2}",
                        service_identifiers.namespace,
                        service_identifiers.name,
                        service_identifiers.version
                    );
                    warn!(not_found_message);
                    Err(Status::not_found(not_found_message))
                }
            }
        }
    }

    /// Inspects the contents of the service registry.
    ///
    /// This function retrieves all services currently registered based on an [`InspectRequest`]. Returns a
    /// [`InspectResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - A [`InspectRequest`] wrapped by a [`tonic::Request`].
    async fn inspect(
        &self,
        _request: Request<InspectRequest>,
    ) -> Result<Response<InspectResponse>, Status> {
        let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
            self.registry_map.read();
        let services_list = lock.values().cloned().collect();
        let inspect_response = InspectResponse { services: services_list };
        Ok(Response::new(inspect_response))
    }
}

/// Identifiers for a given service.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceIdentifiers {
    /// namespace represents a logical grouping of services
    namespace: String,
    /// the service name (without the namespace)
    name: String,
    /// the version of the service
    version: String,
}

#[cfg(test)]
mod registry_impl_test {
    use super::*;

    fn has_service(
        registry_map: Arc<RwLock<HashMap<ServiceIdentifiers, ServiceMetadata>>>,
        key: &ServiceIdentifiers,
    ) -> bool {
        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                registry_map.read();
            lock.contains_key(key)
        }
    }

    #[tokio::test]
    async fn register_test() {
        // Test creating a new registration
        let mut service1 = ServiceMetadata {
            namespace: String::from("sdv.test"),
            name: String::from("test_service"),
            version: String::from("1.0.0.0"),
            uri: String::from("localhost:1000"),
            communication_kind: String::from("grpc+proto"),
            communication_reference: String::from("sdv.test.test_service.v1.proto"),
        };

        let registry_map = Arc::new(RwLock::new(HashMap::new()));
        let registry_impl = RegistryImpl { registry_map };
        let request = tonic::Request::new(RegisterRequest { service: Some(service1.clone()) });
        let result = registry_impl.register(request).await;
        assert!(result.is_ok(), "register result is not okay: {result:?}");
        assert_eq!(
            result.unwrap().into_inner().registration_status.clone(),
            RegistrationStatus::NewlyRegistered as i32
        );
        let service_identifiers = ServiceIdentifiers {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };
        assert!(
            has_service(registry_impl.registry_map.clone(), &service_identifiers),
            "service not present in registry"
        );

        // Test updating a registration
        service1.uri = String::from("localhost:1001");
        let update_request =
            tonic::Request::new(RegisterRequest { service: Some(service1.clone()) });
        let updated_result = registry_impl.register(update_request).await;

        assert!(updated_result.is_ok(), "register result is not okay: {updated_result:?}");
        assert_eq!(
            updated_result.unwrap().into_inner().registration_status.clone(),
            RegistrationStatus::Updated as i32
        );
        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                registry_impl.registry_map.read();
            let updated_service_result = lock.get(&service_identifiers);
            assert_eq!(updated_service_result.unwrap().uri, String::from("localhost:1001"));
        }
    }

    #[tokio::test]
    async fn unregister_test() {
        let registry_map = Arc::new(RwLock::new(HashMap::new()));

        let service1 = ServiceMetadata {
            namespace: String::from("sdv.test"),
            name: String::from("test_service"),
            version: String::from("1.0.0.0"),
            uri: String::from("localhost:1000"),
            communication_kind: String::from("grpc+proto"),
            communication_reference: String::from("sdv.test.test_service.v1.proto"),
        };
        let service_identifiers1 = ServiceIdentifiers {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                registry_map.write();
            lock.insert(service_identifiers1.clone(), service1.clone());
        }

        let registry_impl = RegistryImpl { registry_map };

        // Unregister Service
        let request = tonic::Request::new(UnregisterRequest {
            namespace: service_identifiers1.namespace.clone(),
            name: service_identifiers1.name.clone(),
            version: service_identifiers1.version.clone(),
        });
        let result = registry_impl.unregister(request).await;
        assert!(result.is_ok(), "Unregister result is not okay: {result:?}");

        // Unregister Service that doesn't exist
        let request2 = tonic::Request::new(UnregisterRequest {
            namespace: service_identifiers1.namespace.clone(),
            name: service_identifiers1.name.clone(),
            version: service_identifiers1.version.clone(),
        });
        let not_found_status = Status::not_found(format!(
            "Unable to remove service from registry: {:?}",
            service_identifiers1
        ));
        let result = registry_impl.unregister(request2).await.err().unwrap();
        assert_eq!(result.code(), not_found_status.code());
        assert_eq!(result.message(), not_found_status.message());
    }

    #[tokio::test]
    async fn discover_test() {
        let registry_map = Arc::new(RwLock::new(HashMap::new()));

        let service1 = ServiceMetadata {
            namespace: String::from("sdv.test"),
            name: String::from("test_service"),
            version: String::from("1.0.0.0"),
            uri: String::from("localhost:1000"),
            communication_kind: String::from("grpc+proto"),
            communication_reference: String::from("sdv.test.test_service.v1.proto"),
        };
        let service_identifiers1 = ServiceIdentifiers {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                registry_map.write();
            lock.insert(service_identifiers1.clone(), service1.clone());
        }

        let registry_impl = RegistryImpl { registry_map };

        // Discover Service
        let request = tonic::Request::new(DiscoverServiceRequest {
            namespace: service_identifiers1.namespace.clone(),
            name: service_identifiers1.name.clone(),
            version: service_identifiers1.version.clone(),
        });
        let result = registry_impl.discover_service(request).await;
        assert!(result.is_ok(), "DiscoverService result is not okay: {result:?}");
        assert_eq!(result.unwrap().into_inner().service, Some(service1.clone()));

        // Discover by namespace
        let request_namespace = tonic::Request::new(DiscoverByNamespaceRequest {
            namespace: service_identifiers1.namespace.clone(),
        });
        let result_namespace = registry_impl.discover_by_namespace(request_namespace).await;
        assert!(
            result_namespace.is_ok(),
            "DiscoverByNamespace result is not okay: {result_namespace:?}"
        );
        assert_eq!(result_namespace.unwrap().into_inner().services[0], service1.clone());
    }

    #[tokio::test]
    async fn inspect_test() {
        let registry_map = Arc::new(RwLock::new(HashMap::new()));

        let service1 = ServiceMetadata {
            namespace: String::from("sdv.test"),
            name: String::from("test_service"),
            version: String::from("1.0.0.0"),
            uri: String::from("localhost:1000"),
            communication_kind: String::from("grpc+proto"),
            communication_reference: String::from("sdv.test.test_service.v1.proto"),
        };
        let service2 = ServiceMetadata {
            namespace: String::from("sdv.test"),
            name: String::from("test_service"),
            version: String::from("2.0.0.0"),
            uri: String::from("localhost:2000"),
            communication_kind: String::from("grpc+proto"),
            communication_reference: String::from("sdv.test.test_service.v2.proto"),
        };
        let service_identifiers1 = ServiceIdentifiers {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };
        let service_identifiers2 = ServiceIdentifiers {
            namespace: service2.namespace.clone(),
            name: service2.name.clone(),
            version: service2.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                registry_map.write();
            lock.insert(service_identifiers1.clone(), service1.clone());
            lock.insert(service_identifiers2.clone(), service2.clone());
        }

        let registry_impl = RegistryImpl { registry_map };

        // Test that inspect returns the two services
        let request = tonic::Request::new(InspectRequest {});
        let result = registry_impl.inspect(request).await;
        assert!(result.is_ok(), "Inspect result is not okay: {result:?}");
        let result_services = result.unwrap().into_inner().services;
        assert_eq!(result_services.len(), 2);
        assert!(
            result_services.contains(&service1),
            "Service1 not present in the inspect response"
        );
        assert!(
            result_services.contains(&service2),
            "Service2 not present in the inspect response"
        );
    }
}
