use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::update_details::project_items_update_details_response::ProjectItemsUpdateDetailsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::details::{DetailsFieldSource, DetailsValue};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectItemsUpdateDetailsRequest {
    pub project_item_paths: Vec<PathBuf>,

    #[serde(default)]
    pub property_name: Option<String>,

    #[serde(default)]
    pub address_target_property_name: Option<String>,

    #[serde(default)]
    pub anonymous_value_string: Option<AnonymousValueString>,

    #[serde(default)]
    pub details_field_source: DetailsFieldSource,

    #[serde(default)]
    pub details_value: DetailsValue,
}

impl ProjectItemsUpdateDetailsRequest {
    pub fn from_details_update(
        project_item_paths: Vec<PathBuf>,
        details_field_source: DetailsFieldSource,
        details_value: DetailsValue,
    ) -> Self {
        Self {
            project_item_paths,
            details_field_source,
            details_value,
            ..Default::default()
        }
    }

    pub fn resolve_details_update(&self) -> Result<(DetailsFieldSource, DetailsValue), String> {
        if self.details_field_source != DetailsFieldSource::Unknown {
            return Ok((self.details_field_source.clone(), self.details_value.clone()));
        }

        let details_field_source = match (&self.property_name, &self.address_target_property_name) {
            (Some(property_name), None) => DetailsFieldSource::ProjectItemProperty {
                property_name: property_name.clone(),
            },
            (None, Some(property_name)) => DetailsFieldSource::ProjectItemAddressTarget {
                property_name: property_name.clone(),
            },
            (Some(_), Some(_)) => {
                return Err(String::from(
                    "Project item details update must target either a stored property or an address target property, not both.",
                ));
            }
            (None, None) => return Err(String::from("Project item details update target is required.")),
        };
        let details_value = self
            .anonymous_value_string
            .clone()
            .map(DetailsValue::AnonymousValue)
            .unwrap_or(DetailsValue::Empty);

        Ok((details_field_source, details_value))
    }
}

impl UnprivilegedCommandRequest for ProjectItemsUpdateDetailsRequest {
    type ResponseType = ProjectItemsUpdateDetailsResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::UpdateDetails {
            project_items_update_details_request: self.clone(),
        })
    }
}

impl From<ProjectItemsUpdateDetailsResponse> for ProjectItemsResponse {
    fn from(project_items_update_details_response: ProjectItemsUpdateDetailsResponse) -> Self {
        ProjectItemsResponse::UpdateDetails {
            project_items_update_details_response,
        }
    }
}
