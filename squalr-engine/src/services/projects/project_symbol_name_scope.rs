use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
};
use std::collections::HashSet;

pub struct ProjectSymbolNameScope;

impl ProjectSymbolNameScope {
    pub fn deduplicate_display_name(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_claims: &[ProjectSymbolClaim],
        locator: &ProjectSymbolLocator,
        requested_display_name: &str,
        ignored_locator_key: Option<&str>,
    ) -> String {
        let trimmed_display_name = requested_display_name.trim();
        let base_display_name = if trimmed_display_name.is_empty() { "Symbol" } else { trimmed_display_name };
        let reserved_display_names = Self::collect_reserved_display_names(project_symbol_catalog, symbol_claims, locator, ignored_locator_key);

        if !reserved_display_names.contains(base_display_name) {
            return base_display_name.to_string();
        }

        let mut duplicate_sequence_number = 0_u64;
        loop {
            let candidate_display_name = format!("{}_{}", base_display_name, duplicate_sequence_number);

            if !reserved_display_names.contains(candidate_display_name.as_str()) {
                return candidate_display_name;
            }

            duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
        }
    }

    fn collect_reserved_display_names(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_claims: &[ProjectSymbolClaim],
        locator: &ProjectSymbolLocator,
        ignored_locator_key: Option<&str>,
    ) -> HashSet<String> {
        let mut reserved_display_names = HashSet::new();

        match locator {
            ProjectSymbolLocator::AbsoluteAddress { .. } => {
                for symbol_claim in symbol_claims {
                    if ignored_locator_key == Some(symbol_claim.get_symbol_locator_key().as_str()) {
                        continue;
                    }

                    if matches!(symbol_claim.get_locator(), ProjectSymbolLocator::AbsoluteAddress { .. }) {
                        reserved_display_names.insert(symbol_claim.get_display_name().to_string());
                    }
                }
            }
            ProjectSymbolLocator::ModuleOffset { module_name, .. } => {
                if let Some(symbol_module) = project_symbol_catalog.find_symbol_module(module_name) {
                    for module_field in symbol_module.get_fields() {
                        if ignored_locator_key == Some(module_field.get_symbol_locator_key(module_name).as_str()) {
                            continue;
                        }

                        reserved_display_names.insert(module_field.get_display_name().to_string());
                    }
                }

                for symbol_claim in symbol_claims {
                    if ignored_locator_key == Some(symbol_claim.get_symbol_locator_key().as_str()) {
                        continue;
                    }

                    let ProjectSymbolLocator::ModuleOffset {
                        module_name: symbol_claim_module_name,
                        ..
                    } = symbol_claim.get_locator()
                    else {
                        continue;
                    };

                    if symbol_claim_module_name == module_name {
                        reserved_display_names.insert(symbol_claim.get_display_name().to_string());
                    }
                }
            }
        }

        reserved_display_names
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolNameScope;
    use squalr_engine_api::structures::projects::{
        project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
    };

    #[test]
    fn deduplicate_display_name_reserves_module_fields_and_claims_in_same_module() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Timer"), 0x00, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Timer_0"),
                String::from("game.exe"),
                0x08,
                String::from("u32"),
            )],
        );
        let display_name = ProjectSymbolNameScope::deduplicate_display_name(
            &project_symbol_catalog,
            project_symbol_catalog.get_symbol_claims(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x10),
            "Timer",
            None,
        );

        assert_eq!(display_name, "Timer_1");
    }

    #[test]
    fn deduplicate_display_name_ignores_current_locator() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Timer"),
                0x1234,
                String::from("u32"),
            )],
        );
        let display_name = ProjectSymbolNameScope::deduplicate_display_name(
            &project_symbol_catalog,
            project_symbol_catalog.get_symbol_claims(),
            &ProjectSymbolLocator::new_absolute_address(0x1234),
            "Timer",
            Some("absolute:1234"),
        );

        assert_eq!(display_name, "Timer");
    }
}
