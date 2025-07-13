//! MCP prompts for barefoot runner

use super::*;
use std::collections::HashMap;
use crate::error::Result;

/// Prompt template manager
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Add default prompt templates
        templates.insert(
            "job_failure_analysis".to_string(),
            PromptTemplate {
                name: "job_failure_analysis".to_string(),
                description: "Analyze a failed job and suggest fixes".to_string(),
                content: "Analyze the following failed job and suggest potential fixes:\n\nJob ID: {job_id}\nStatus: {status}\nLogs: {logs}\nDuration: {duration}\n\nWhat went wrong and how can it be fixed? Consider:\n1. Common failure patterns\n2. Configuration issues\n3. Resource constraints\n4. Dependencies or timing issues\n\nProvide specific, actionable recommendations.".to_string(),
                variables: vec!["job_id".to_string(), "status".to_string(), "logs".to_string(), "duration".to_string()],
                category: "troubleshooting".to_string(),
            },
        );
        
        templates.insert(
            "performance_optimization".to_string(),
            PromptTemplate {
                name: "performance_optimization".to_string(),
                description: "Analyze job performance and suggest optimizations".to_string(),
                content: "Analyze the performance of the following job and suggest optimizations:\n\nJob ID: {job_id}\nDuration: {duration}\nSteps: {steps}\nResources: {resources}\n\nWhat optimizations could improve performance? Consider:\n1. Parallel execution opportunities\n2. Resource allocation\n3. Caching strategies\n4. Step optimization\n5. Dependency optimization\n\nProvide specific, measurable recommendations.".to_string(),
                variables: vec!["job_id".to_string(), "duration".to_string(), "steps".to_string(), "resources".to_string()],
                category: "optimization".to_string(),
            },
        );
        
        templates.insert(
            "job_scheduling".to_string(),
            PromptTemplate {
                name: "job_scheduling".to_string(),
                description: "Help schedule jobs based on dependencies and resources".to_string(),
                content: "Help schedule the following jobs based on their dependencies and available resources:\n\nJobs: {jobs}\nDependencies: {dependencies}\nResources: {resources}\nConstraints: {constraints}\n\nWhat is the optimal execution order? Consider:\n1. Dependency relationships\n2. Resource availability\n3. Priority levels\n4. Parallel execution opportunities\n5. Resource conflicts\n\nProvide a detailed execution plan with timing estimates.".to_string(),
                variables: vec!["jobs".to_string(), "dependencies".to_string(), "resources".to_string(), "constraints".to_string()],
                category: "scheduling".to_string(),
            },
        );
        
        templates.insert(
            "configuration_validation".to_string(),
            PromptTemplate {
                name: "configuration_validation".to_string(),
                description: "Validate runner configuration and suggest improvements".to_string(),
                content: "Validate the following runner configuration and suggest improvements:\n\nConfiguration: {config}\nCurrent Issues: {issues}\nPerformance Metrics: {metrics}\n\nWhat improvements can be made? Consider:\n1. Resource allocation\n2. Security settings\n3. Performance tuning\n4. Best practices\n5. Scalability concerns\n\nProvide specific configuration recommendations.".to_string(),
                variables: vec!["config".to_string(), "issues".to_string(), "metrics".to_string()],
                category: "configuration".to_string(),
            },
        );
        
        templates.insert(
            "error_pattern_analysis".to_string(),
            PromptTemplate {
                name: "error_pattern_analysis".to_string(),
                description: "Analyze error patterns across multiple job failures".to_string(),
                content: "Analyze the following error patterns across multiple job failures:\n\nError Logs: {error_logs}\nJob Types: {job_types}\nTime Period: {time_period}\nFrequency: {frequency}\n\nWhat patterns do you observe? Consider:\n1. Common error types\n2. Temporal patterns\n3. Job-specific issues\n4. System-level problems\n5. Root cause analysis\n\nProvide insights and preventive measures.".to_string(),
                variables: vec!["error_logs".to_string(), "job_types".to_string(), "time_period".to_string(), "frequency".to_string()],
                category: "analysis".to_string(),
            },
        );
        
        templates.insert(
            "resource_allocation".to_string(),
            PromptTemplate {
                name: "resource_allocation".to_string(),
                description: "Optimize resource allocation for job execution".to_string(),
                content: "Optimize resource allocation for the following job execution scenario:\n\nCurrent Jobs: {current_jobs}\nAvailable Resources: {available_resources}\nJob Requirements: {job_requirements}\nPerformance Goals: {performance_goals}\n\nHow should resources be allocated? Consider:\n1. Job priorities\n2. Resource efficiency\n3. Fairness\n4. Performance optimization\n5. Resource constraints\n\nProvide a detailed allocation strategy.".to_string(),
                variables: vec!["current_jobs".to_string(), "available_resources".to_string(), "job_requirements".to_string(), "performance_goals".to_string()],
                category: "optimization".to_string(),
            },
        );
        
        templates.insert(
            "monitoring_setup".to_string(),
            PromptTemplate {
                name: "monitoring_setup".to_string(),
                description: "Design monitoring and alerting for the runner system".to_string(),
                content: "Design monitoring and alerting for the following runner system:\n\nSystem Components: {components}\nCurrent Metrics: {metrics}\nAlert Requirements: {alert_requirements}\nPerformance Targets: {performance_targets}\n\nWhat monitoring should be implemented? Consider:\n1. Key performance indicators\n2. Alert thresholds\n3. Dashboard design\n4. Notification channels\n5. Escalation procedures\n\nProvide a comprehensive monitoring strategy.".to_string(),
                variables: vec!["components".to_string(), "metrics".to_string(), "alert_requirements".to_string(), "performance_targets".to_string()],
                category: "monitoring".to_string(),
            },
        );
        
        templates.insert(
            "security_audit".to_string(),
            PromptTemplate {
                name: "security_audit".to_string(),
                description: "Perform a security audit of the runner system".to_string(),
                content: "Perform a security audit of the following runner system:\n\nSystem Architecture: {architecture}\nCurrent Security: {current_security}\nAccess Patterns: {access_patterns}\nThreat Model: {threat_model}\n\nWhat security improvements are needed? Consider:\n1. Authentication mechanisms\n2. Authorization controls\n3. Data protection\n4. Network security\n5. Audit logging\n\nProvide a comprehensive security assessment.".to_string(),
                variables: vec!["architecture".to_string(), "current_security".to_string(), "access_patterns".to_string(), "threat_model".to_string()],
                category: "security".to_string(),
            },
        );
        
        Self { templates }
    }
    
    /// Get a prompt template by name
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }
    
    /// List all available prompt templates
    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
    
    /// List templates by category
    pub fn list_templates_by_category(&self, category: &str) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|template| template.category == category)
            .collect()
    }
    
    /// Render a prompt template with variables
    pub fn render_template(&self, name: &str, variables: HashMap<String, String>) -> Result<String> {
        if let Some(template) = self.get_template(name) {
            let mut content = template.content.clone();
            
            for (var_name, var_value) in variables {
                let placeholder = format!("{{{}}}", var_name);
                content = content.replace(&placeholder, &var_value);
            }
            
            Ok(content)
        } else {
            Err(BarefootError::Mcp(format!("Prompt template not found: {}", name)))
        }
    }
    
    /// Add a custom prompt template
    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    /// Remove a prompt template
    pub fn remove_template(&mut self, name: &str) -> Option<PromptTemplate> {
        self.templates.remove(name)
    }
    
    /// Get template categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        for template in self.templates.values() {
            categories.insert(template.category.clone());
        }
        categories.into_iter().collect()
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt template builder
pub struct PromptTemplateBuilder {
    name: String,
    description: String,
    content: String,
    variables: Vec<String>,
    category: String,
}

impl PromptTemplateBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            content: String::new(),
            variables: Vec::new(),
            category: "custom".to_string(),
        }
    }
    
    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    pub fn content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
    
    pub fn variable(mut self, variable: String) -> Self {
        self.variables.push(variable);
        self
    }
    
    pub fn variables(mut self, variables: Vec<String>) -> Self {
        self.variables = variables;
        self
    }
    
    pub fn category(mut self, category: String) -> Self {
        self.category = category;
        self
    }
    
    pub fn build(self) -> PromptTemplate {
        PromptTemplate {
            name: self.name,
            description: self.description,
            content: self.content,
            variables: self.variables,
            category: self.category,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_manager_creation() {
        let manager = PromptManager::new();
        
        assert!(manager.get_template("job_failure_analysis").is_some());
        assert!(manager.get_template("performance_optimization").is_some());
        assert!(manager.get_template("nonexistent").is_none());
    }
    
    #[test]
    fn test_prompt_template_builder() {
        let template = PromptTemplateBuilder::new("test_template".to_string())
            .description("Test description".to_string())
            .content("Test content with {variable}".to_string())
            .variable("variable".to_string())
            .category("test".to_string())
            .build();
        
        assert_eq!(template.name, "test_template");
        assert_eq!(template.description, "Test description");
        assert_eq!(template.content, "Test content with {variable}");
        assert_eq!(template.variables, vec!["variable"]);
        assert_eq!(template.category, "test");
    }
    
    #[test]
    fn test_template_rendering() {
        let manager = PromptManager::new();
        let mut variables = HashMap::new();
        variables.insert("job_id".to_string(), "test-job-123".to_string());
        variables.insert("status".to_string(), "failed".to_string());
        variables.insert("logs".to_string(), "Error: test error".to_string());
        variables.insert("duration".to_string(), "30s".to_string());
        
        let rendered = manager.render_template("job_failure_analysis", variables).unwrap();
        assert!(rendered.contains("test-job-123"));
        assert!(rendered.contains("failed"));
        assert!(rendered.contains("Error: test error"));
        assert!(rendered.contains("30s"));
    }
    
    #[test]
    fn test_template_categories() {
        let manager = PromptManager::new();
        let categories = manager.get_categories();
        
        assert!(categories.contains(&"troubleshooting".to_string()));
        assert!(categories.contains(&"optimization".to_string()));
        assert!(categories.contains(&"scheduling".to_string()));
    }
    
    #[test]
    fn test_custom_template() {
        let mut manager = PromptManager::new();
        
        let custom_template = PromptTemplate {
            name: "custom_template".to_string(),
            description: "Custom template".to_string(),
            content: "Custom content".to_string(),
            variables: vec![],
            category: "custom".to_string(),
        };
        
        manager.add_template(custom_template);
        assert!(manager.get_template("custom_template").is_some());
        
        manager.remove_template("custom_template");
        assert!(manager.get_template("custom_template").is_none());
    }
} 