// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Module containing gRPC service implementation based on [`service_discovery_proto::service_registry::v1`].
//!
//! Provides a gRPC endpoint for external services to interact with to register and discover
//! services. Note: Identifiers in all Registry operations are case-sensitive.

use parking_lot::RwLock;
use service_discovery_proto::service_registry::v1::service_registry_server::ServiceRegistry;
use service_discovery_proto::service_registry::v1::{
    DiscoverByNamespaceRequest, DiscoverByNamespaceResponse, DiscoverRequest, DiscoverResponse,
    ListRequest, ListResponse, RegisterRequest, RegisterResponse, ServiceMetadata,
    UnregisterRequest, UnregisterResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Base structure for the service registry gRPC service
#[derive(Clone, Debug)]
pub struct ServiceRegistryImpl {
    registry_map: Arc<RwLock<HashMap<ServiceIdentifier, ServiceMetadata>>>,
}

impl ServiceRegistryImpl {
    pub fn new(
        registry_map: Arc<RwLock<HashMap<ServiceIdentifier, ServiceMetadata>>>,
    ) -> ServiceRegistryImpl {
        ServiceRegistryImpl { registry_map }
    }
}

#[tonic::async_trait]
impl ServiceRegistry for ServiceRegistryImpl {
    /// Registers a service by adding it to the service registry.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the necessary metadata for the service to be registered
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request_inner = request.into_inner();
        let service_to_register =
            request_inner.service.ok_or_else(|| Status::invalid_argument("Service is required"))?;
        let service_identifier = ServiceIdentifier {
            namespace: service_to_register.namespace.clone(),
            name: service_to_register.name.clone(),
            version: service_to_register.version.clone(),
        };
        info!("Received a register request for: {service_identifier:?}");

        // This block controls the lifetime of the lock.
        {
            let mut lock = self.registry_map.write();
            match lock.get(&service_identifier) {
                Some(existing_service) => {
                    let error_message = format!("Register failed. A service already exists with the same service identifier: {existing_service:?}");
                    warn!(error_message);
                    Err(Status::already_exists(error_message))
                }
                None => {
                    lock.insert(service_identifier.clone(), service_to_register.clone());
                    info!(
                        "Registered new service in the service registry: {service_to_register:?}"
                    );
                    Ok(Response::new(RegisterResponse {}))
                }
            }
        }
    }

    /// Unregisters a service by removing it from the registry.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the service identification information for the service to unregister
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        let request_inner = request.into_inner();
        let service_identifier = ServiceIdentifier {
            namespace: request_inner.namespace,
            name: request_inner.name,
            version: request_inner.version,
        };
        info!("Received an unregister request for: {service_identifier:?}");

        let mut lock = self.registry_map.write();
        match lock.remove(&service_identifier) {
            Some(removed_service) => {
                info!(
                    "Successfully removed service entry from service registry: {removed_service:?}"
                );
                Ok(Response::new(UnregisterResponse {}))
            }
            None => {
                let not_found_message =
                    format!("Unable to remove service from registry: {service_identifier:?}");
                warn!(not_found_message);
                Err(Status::not_found(not_found_message))
            }
        }
    }

    /// Discovers a list of services based on the namespace, or logical grouping of services.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the namespace of the services to be discovered
    async fn discover_by_namespace(
        &self,
        request: Request<DiscoverByNamespaceRequest>,
    ) -> Result<Response<DiscoverByNamespaceResponse>, Status> {
        let namespace = request.into_inner().namespace;

        // This block controls the lifetime of the lock.
        let service_list: Vec<ServiceMetadata> = {
            let lock = self.registry_map.read();
            lock.iter()
                .filter_map(|(service_identifier, service_metadata)| {
                    (service_identifier.namespace == namespace).then(|| service_metadata.clone())
                })
                .collect()
        };
        let discover_by_namespace_response = DiscoverByNamespaceResponse { services: service_list };
        Ok(Response::new(discover_by_namespace_response))
    }

    /// Discovers a single service based on its "fully qualified name", consisting of the namespace,
    /// name, and version of the service.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the service identification information for the service to discover
    async fn discover(
        &self,
        request: Request<DiscoverRequest>,
    ) -> Result<Response<DiscoverResponse>, Status> {
        let request_inner = request.into_inner();

        let service_identifier = ServiceIdentifier {
            namespace: request_inner.namespace.clone(),
            name: request_inner.name.clone(),
            version: request_inner.version,
        };

        // This block controls the lifetime of the lock.
        let service_option = {
            let lock = self.registry_map.read();
            lock.get(&service_identifier).cloned()
        };

        match service_option {
            Some(service) => {
                info!("Read service in Discover {service:?}");
                let discover_response = DiscoverResponse { service: Some(service) };
                Ok(Response::new(discover_response))
            }
            None => {
                let not_found_message = format!(
                    "No service found for namespace: {0}, name: {1}, version: {2}",
                    service_identifier.namespace,
                    service_identifier.name,
                    service_identifier.version
                );
                warn!(not_found_message);
                Err(Status::not_found(not_found_message))
            }
        }
    }

    /// Lists all services currently registered with the service registry.
    ///
    /// # Arguments
    ///
    /// * `request` - Empty ListRequest
    async fn list(&self, _request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let lock = self.registry_map.read();
        let services_list = lock.values().cloned().collect();
        let list_response = ListResponse { services: services_list };
        Ok(Response::new(list_response))
    }
}

/// Identifiers for a given service.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceIdentifier {
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
        registry_map: Arc<RwLock<HashMap<ServiceIdentifier, ServiceMetadata>>>,
        key: &ServiceIdentifier,
    ) -> bool {
        // This block controls the lifetime of the lock.
        {
            let lock = registry_map.read();
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
        let registry_impl = ServiceRegistryImpl { registry_map };
        let request = tonic::Request::new(RegisterRequest { service: Some(service1.clone()) });
        let result = registry_impl.register(request).await;
        assert!(result.is_ok(), "register result is not okay: {result:?}");

        let service_identifier = ServiceIdentifier {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };
        assert!(
            has_service(registry_impl.registry_map.clone(), &service_identifier),
            "service not present in registry"
        );

        // Test adding a registration with same identifier fails
        service1.uri = String::from("localhost:1001");
        let existing_service_request =
            tonic::Request::new(RegisterRequest { service: Some(service1.clone()) });
        let existing_service_result = registry_impl.register(existing_service_request).await;

        assert!(
            existing_service_result.is_err(),
            "Registering an existing service should fail: {existing_service_result:?}"
        );
        assert_eq!(existing_service_result.unwrap_err().code(), Status::already_exists("").code());
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
        let service_identifier1 = ServiceIdentifier {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock = registry_map.write();
            lock.insert(service_identifier1.clone(), service1.clone());
        }

        let registry_impl = ServiceRegistryImpl { registry_map };

        // Unregister Service
        let request = tonic::Request::new(UnregisterRequest {
            namespace: service_identifier1.namespace.clone(),
            name: service_identifier1.name.clone(),
            version: service_identifier1.version.clone(),
        });
        let result = registry_impl.unregister(request).await;
        assert!(result.is_ok(), "Unregister result is not okay: {result:?}");

        // Unregister Service that doesn't exist
        let request2 = tonic::Request::new(UnregisterRequest {
            namespace: service_identifier1.namespace.clone(),
            name: service_identifier1.name.clone(),
            version: service_identifier1.version.clone(),
        });
        let not_found_status = Status::not_found(format!(
            "Unable to remove service from registry: {service_identifier1:?}"
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
        let service_identifier1 = ServiceIdentifier {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock = registry_map.write();
            lock.insert(service_identifier1.clone(), service1.clone());
        }

        let registry_impl = ServiceRegistryImpl { registry_map };

        // Discover Service
        let request = tonic::Request::new(DiscoverRequest {
            namespace: service_identifier1.namespace.clone(),
            name: service_identifier1.name.clone(),
            version: service_identifier1.version.clone(),
        });
        let result = registry_impl.discover(request).await;
        assert!(result.is_ok(), "Discover result is not okay: {result:?}");
        assert_eq!(result.unwrap().into_inner().service, Some(service1.clone()));

        // Discover by namespace
        let request_namespace = tonic::Request::new(DiscoverByNamespaceRequest {
            namespace: service_identifier1.namespace.clone(),
        });
        let result_namespace = registry_impl.discover_by_namespace(request_namespace).await;
        assert!(
            result_namespace.is_ok(),
            "DiscoverByNamespace result is not okay: {result_namespace:?}"
        );
        assert_eq!(result_namespace.unwrap().into_inner().services[0], service1.clone());

        // Discover by namespace that has no services
        let request_namespace_none =
            tonic::Request::new(DiscoverByNamespaceRequest { namespace: String::from("sdv.none") });
        let result_namespace_none =
            registry_impl.discover_by_namespace(request_namespace_none).await;
        assert!(
            result_namespace_none.is_ok(),
            "DiscoverByNamespace with no services result is not okay: {result_namespace_none:?}"
        );
        assert!(result_namespace_none.unwrap().into_inner().services.is_empty());
    }

    #[tokio::test]
    async fn list_test() {
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
        let service_identifier1 = ServiceIdentifier {
            namespace: service1.namespace.clone(),
            name: service1.name.clone(),
            version: service1.version.clone(),
        };
        let service_identifier2 = ServiceIdentifier {
            namespace: service2.namespace.clone(),
            name: service2.name.clone(),
            version: service2.version.clone(),
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock = registry_map.write();
            lock.insert(service_identifier1.clone(), service1.clone());
            lock.insert(service_identifier2.clone(), service2.clone());
        }

        let registry_impl = ServiceRegistryImpl { registry_map };

        // Test that list returns the two services
        let request = tonic::Request::new(ListRequest {});
        let result = registry_impl.list(request).await;
        assert!(result.is_ok(), "List result is not okay: {result:?}");
        let result_services = result.unwrap().into_inner().services;
        assert_eq!(result_services.len(), 2);
        assert!(result_services.contains(&service1), "Service1 not present in the list response");
        assert!(result_services.contains(&service2), "Service2 not present in the list response");
    }
}
