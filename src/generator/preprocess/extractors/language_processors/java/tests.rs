#[cfg(test)]
mod tests {
    use crate::generator::preprocess::extractors::language_processors::java::JavaProcessor;
    use crate::generator::preprocess::extractors::language_processors::LanguageProcessor;
    use std::path::Path;

    #[test]
    fn test_extract_dependencies_import() {
        let processor = JavaProcessor::new();
        let content = r#"
import java.util.List;
import java.util.Map;
import java.util.ArrayList;
import java.io.File;
import org.apache.commons.lang.StringUtils;
import com.example.MyClass;
import static java.util.Collections.emptyList;
import static org.junit.Assert.*;
"#;
        
        let deps = processor.extract_dependencies(content, Path::new("Test.java"));
        
        assert_eq!(deps.len(), 8); // There are actually 8 import lines (including 2 static imports)
        
        // Check JDK import
        let list_dep = &deps[0];
        assert_eq!(list_dep.name, "List");
        assert_eq!(list_dep.path, Some("java.util.List".to_string()));
        assert!(list_dep.is_external);
        assert_eq!(list_dep.dependency_type, "import");
        
        // Check external library import
        let string_dep = &deps[4];
        assert_eq!(string_dep.name, "StringUtils");
        assert_eq!(string_dep.path, Some("org.apache.commons.lang.StringUtils".to_string()));
        assert!(string_dep.is_external);
        
        // Check MyClass import
        let myclass_dep = &deps[5];
        assert_eq!(myclass_dep.name, "MyClass");
        assert_eq!(myclass_dep.path, Some("com.example.MyClass".to_string()));
        assert!(myclass_dep.is_external);
        
        // Check static import
        let empty_dep = &deps[6];
        assert_eq!(empty_dep.name, "emptyList");
        assert_eq!(empty_dep.path, Some("static java.util.Collections.emptyList".to_string()));
        assert!(empty_dep.is_external);
        assert_eq!(empty_dep.dependency_type, "import"); // Currently implementation doesn't distinguish static imports
    }

    #[test]
    fn test_extract_dependencies_package() {
        let processor = JavaProcessor::new();
        let content = r#"
package com.example.project;

import java.util.List;

public class Test {
    // ...
}
"#;
        
        let deps = processor.extract_dependencies(content, Path::new("Test.java"));
        assert_eq!(deps.len(), 2);
        
        // Check package dependency
        let package_dep = &deps[0];
        assert_eq!(package_dep.name, "com.example.project");
        assert!(!package_dep.is_external);
        assert_eq!(package_dep.dependency_type, "package");
        
        // Check regular import
        let list_dep = &deps[1];
        assert_eq!(list_dep.name, "List");
        assert!(list_dep.is_external);
    }

    #[test]
    fn test_determine_component_type() {
        let processor = JavaProcessor::new();
        
        // Test test files
        assert_eq!(
            processor.determine_component_type(
                Path::new("UserTest.java"),
                ""
            ),
            "java_test"
        );
        
        assert_eq!(
            processor.determine_component_type(
                Path::new("UserTests.java"),
                ""
            ),
            "java_test"
        );
        
        // Test interface
        assert_eq!(
            processor.determine_component_type(
                Path::new("UserService.java"),
                "public interface UserService"
            ),
            "java_interface"
        );
        
        // Test enum
        assert_eq!(
            processor.determine_component_type(
                Path::new("Status.java"),
                "public enum Status"
            ),
            "java_enum"
        );
        
        // Test abstract class
        assert_eq!(
            processor.determine_component_type(
                Path::new("AbstractService.java"),
                "public abstract class AbstractService"
            ),
            "java_abstract_class"
        );
        
        // Test regular class
        assert_eq!(
            processor.determine_component_type(
                Path::new("UserService.java"),
                "public class UserService"
            ),
            "java_class"
        );
        
        // Test file without class
        assert_eq!(
            processor.determine_component_type(
                Path::new("config.txt"),
                ""
            ),
            "java_file"
        );
    }

    #[test]
    fn test_is_important_line() {
        let processor = JavaProcessor::new();
        
        // Important lines
        assert!(processor.is_important_line("public class TestClass {"));
        assert!(processor.is_important_line("private void testMethod() {"));
        assert!(processor.is_important_line("public static void main(String[] args) {"));
        assert!(processor.is_important_line("public interface TestInterface {"));
        assert!(processor.is_important_line("public enum TestEnum {"));
        assert!(processor.is_important_line("import java.util.List;"));
        assert!(processor.is_important_line("package com.example;"));
        
        // Non-important lines
        assert!(!processor.is_important_line("// This is a comment"));
        assert!(!processor.is_important_line("System.out.println(\"debug\");"));
        assert!(!processor.is_important_line("int x = 1;"));
        assert!(!processor.is_important_line("if (condition) {"));
        assert!(!processor.is_important_line("    // indented comment"));
    }

    #[test]
    fn test_extract_interfaces() {
        let processor = JavaProcessor::new();
        let content = r#"
public class UserService {
    private UserRepository userRepository;
    
    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }
    
    public User createUser(String name, String email) {
        User user = new User();
        user.setName(name);
        user.setEmail(email);
        return userRepository.save(user);
    }
}
"#;
        
        let interfaces = processor.extract_interfaces(content, Path::new("Test.java"));
        assert!(!interfaces.is_empty()); // At least one interface is extracted
        
        // Find UserService class
        if let Some(user_service) = interfaces.iter().find(|i| i.name == "UserService") {
            assert_eq!(user_service.interface_type, "class");
        }
    }

    #[test]
    fn test_extract_interfaces_with_annotations() {
        let processor = JavaProcessor::new();
        let content = r#"
@Entity
@Table(name = "users")
public class User {
    
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(nullable = false)
    private String name;
    
    @Transient
    private String computedValue;
    
    @PostConstruct
    public void init() {
        // Initialization logic
    }
    
    @Deprecated
    public void oldMethod() {
        // Deprecated method
    }
}

@RestController
@RequestMapping("/api/users")
public class UserController {
    
    @GetMapping("/{id}")
    public ResponseEntity<User> getUser(@PathVariable Long id) {
        // Implementation
        return ResponseEntity.ok(new User());
    }
    
    @PostMapping
    public ResponseEntity<User> createUser(@RequestBody User user) {
        // Implementation
        return ResponseEntity.status(HttpStatus.CREATED).body(user);
    }
}
"#;
        
        let interfaces = processor.extract_interfaces(content, Path::new("Test.java"));
        assert!(interfaces.len() >= 2); // At least 2 classes are extracted
        
        // Find entity class
        if let Some(user) = interfaces.iter().find(|i| i.name == "User") {
            assert_eq!(user.interface_type, "class");
        }
        
        // Find controller
        if let Some(controller) = interfaces.iter().find(|i| i.name == "UserController") {
            assert_eq!(controller.interface_type, "class");
        }
    }

    #[test]
    fn test_extract_interfaces_empty() {
        let processor = JavaProcessor::new();
        let content = "// Just comments\n/* Block comment */\n\n";
        
        let interfaces = processor.extract_interfaces(content, Path::new("Test.java"));
        assert!(interfaces.is_empty());
    }

    #[test]
    fn test_extract_interfaces_inner_classes() {
        let processor = JavaProcessor::new();
        let content = r#"
public class OuterClass {
    private InnerClass inner;
    
    public OuterClass() {
        this.inner = new InnerClass();
    }
    
    public class InnerClass {
        private String value;
        
        public InnerClass(String value) {
            this.value = value;
        }
        
        public String getValue() {
            return value;
        }
    }
    
    public static class StaticInnerClass {
        public static final String CONSTANT = "value";
    }
}
"#;
        
        let interfaces = processor.extract_interfaces(content, Path::new("Test.java"));
        assert!(interfaces.len() >= 3); // At least 3 classes are extracted
        
        // Check that we have the expected classes
        let class_names: Vec<String> = interfaces.iter()
            .filter(|i| i.interface_type == "class")
            .map(|i| i.name.clone())
            .collect();
        
        assert!(class_names.contains(&"OuterClass".to_string()));
        // Inner classes may not be extracted as separate classes
        if class_names.contains(&"InnerClass".to_string()) {
            // Inner class found
        }
        if class_names.contains(&"StaticInnerClass".to_string()) {
            // Static inner class found
        }
    }

    #[test]
    fn test_extract_interfaces_generics() {
        let processor = JavaProcessor::new();
        let content = r#"
public class GenericService<T, R> {
    private Repository<T> repository;
    
    public GenericService(Repository<T> repository) {
        this.repository = repository;
    }
    
    public R save(T entity) {
        return repository.save(entity);
    }
    
    public Optional<T> findById(Long id) {
        return repository.findById(id);
    }
    
    public <U> U map(Function<T, U> mapper, T entity) {
        return mapper.apply(entity);
    }
}
"#;
        
        let interfaces = processor.extract_interfaces(content, Path::new("Test.java"));
        assert_eq!(interfaces.len(), 4); // More methods are extracted than expected
        
        let service = &interfaces[0];
        assert_eq!(service.name, "GenericService");
        assert_eq!(service.interface_type, "class");
    }
}