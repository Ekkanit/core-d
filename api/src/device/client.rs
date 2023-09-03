extern crate rocket;
use std::error::Error;

use storage::database::DatabaseClient;

pub struct DevicesClient {
    database: DatabaseClient,
}

impl DevicesClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }

    pub fn add_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let device_id = device.id.clone();
        let add_device = self.map_device(device);
        let _ = self.database.add_device(add_device)?;
        return self.get_device(device_id.as_str());
    }

    pub fn get_device(&mut self, device_id: &str) -> Result<primitives::device::Device, Box<dyn Error>> {
        let device = self.database.get_device(device_id.to_string())?;
        Ok(
            primitives::device::Device { 
                id: device.device_id, 
                platform: device.platform,
                token: device.token,
                is_push_enabled: device.is_push_enabled,
            }
        )
    }
    pub fn update_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let device_id = device.id.clone();
        let update_device = self.map_device(device);
        let _ = self.database.update_device(update_device)?;
        return self.get_device(device_id.as_str());
    }

    pub fn map_device(&self, device: primitives::device::Device) -> storage::models::Device {
        return storage::models::Device {
            device_id: device.id,
            platform: device.platform,
            token: device.token,
            is_push_enabled: device.is_push_enabled,
        };
    }
}