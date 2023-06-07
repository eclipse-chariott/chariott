// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proto_registry::chariott_registry::v1::registry_server::Registry;
use proto_registry::chariott_registry::v1::{DiscoverServiceRequest, DiscoverServiceResponse, InspectRequest, InspectResponse, RegisterRequest, RegisterResponse, ServiceMetadata};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{collections::HashMap};
use std::sync::Arc;
use tonic::{Status, Request, Response};

#[derive(Clone, Debug)]
pub struct RegistryImpl {
    pub registry_map: Arc<RwLock<HashMap<ServiceIdentifiers, ServiceMetadata>>>,
}

#[tonic::async_trait]
impl Registry for RegistryImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request_inner = request.into_inner();
        let service_to_register = request_inner.service.ok_or_else(|| Status::invalid_argument("service is required"))?;
        let service_identifiers = ServiceIdentifiers {
            namespace: service_to_register.namespace.clone(),
            name: service_to_register.name.clone(),
            version: service_to_register.version.clone()
        };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.write();
            match lock.get(&service_identifiers) {
                Some(_) => {
                    // TODO: Add log that we overwrote the value
                    lock.insert(service_identifiers.clone(), service_to_register.clone());
                }
                None => {
                    // Add log that we added a new entry
                    lock.insert(service_identifiers.clone(), service_to_register.clone());
                }
            };
        }

        
        let register_response = RegisterResponse { };
        Ok(Response::new(register_response))
    }

    async fn discover_service(
        &self,
        request: Request<DiscoverServiceRequest>,
    ) -> Result<Response<DiscoverServiceResponse>, Status> {
        let request_inner = request.into_inner();

        // TODO: check that all of them are included?
        let service_identifiers = ServiceIdentifiers {
            namespace: request_inner.namespace.clone(),
            name: request_inner.name.clone(),
            version: request_inner.version.clone()
        };

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.read();
            match lock.get(&service_identifiers) {
                Some(service) => {
                    // TODO: add log that we read it
                    let discover_service_response = DiscoverServiceResponse { service: Some(service.clone()) };
                    Ok(Response::new(discover_service_response))
                }
                None => {
                    // TODO: add log that it was not found
                    Err(Status::not_found(format!("No service found for namespace: {0}, name: {1}, version: {2}", service_identifiers.namespace, service_identifiers.name, service_identifiers.version)))
                }
            }
        }
    }

    async fn inspect(
        &self,
        _request: Request<InspectRequest>,
    ) -> Result<Response<InspectResponse>, Status> {
        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<ServiceIdentifiers, ServiceMetadata>> =
                self.registry_map.read();
            // transfer ownership to services_list, map can't be used anymore
            let services_list = lock.values().cloned().collect();
            let inspect_response = InspectResponse { services: services_list };
            // TODO: Does all of this need to be in the lock block?
            // TODO: errors???
            Ok(Response::new(inspect_response))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceIdentifiers {
    namespace: String,
    name: String,
    version: String,
}

#[cfg(test)]
mod registry_impl_test {
    #[test]
    fn test_equality() {
        let one = 1;
        assert_eq!(one, 1);
    }
}
