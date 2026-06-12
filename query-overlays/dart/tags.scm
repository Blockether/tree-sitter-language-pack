; Class definitions
(class_definition name: (identifier) @name) @definition.class

; Method declarations inside class bodies
(method_signature name: (identifier) @name) @definition.method
(function_signature name: (identifier) @name) @definition.function

; Class superclass: `class Foo extends Bar`
(class_definition
  superclass: (superclass (type_identifier) @name)) @reference.implementation

; Class interfaces: `class Foo implements Bar, Baz`
(class_definition
  interfaces: (interfaces (type_identifier) @name)) @reference.implementation

; Function / method calls
(function_expression_invocation
  function: (identifier) @name) @reference.call
