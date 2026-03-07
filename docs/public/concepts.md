# Core Concepts

AgenticForge is a blueprint engine that designs complete project architecture
before any code is generated.

## Blueprint

A blueprint is the top-level container. It holds entities, dependencies, file
structures, test cases, and wiring diagrams. Blueprints progress through
statuses: `draft` -> `in_progress` -> `complete` -> `validated` -> `exported`.

## Entity

An entity is a domain object (e.g., User, Order, Task). Entities have fields,
relationships, and validation rules. The EntityInferrer can extract entities
from natural language descriptions automatically.

## Operation

An operation is an action that entities participate in (e.g., CreateUser,
AssignTask). Operations have signatures, error flows, and async annotations.

## Dependency

A dependency is an external library or crate required by the blueprint. The
DependencyInferrer resolves dependencies based on domain and entity analysis.

## Structure

The file structure maps entities and operations to source files and directories.
FileStructureGenerator produces the layout; ImportGraphGenerator resolves
inter-file imports.

## Skeleton

A skeleton is a compilable code stub for an entity or operation. Skeletons
include type definitions, trait impls, and placeholder logic.

## Domain

Domains classify blueprints: `web`, `api`, `cli`, `library`, `service`,
`database`, `embedded`, `mobile`, `desktop`, `plugin`. Each domain influences
layer decomposition and dependency inference.

## Invention Tiers

The 32 inventions are organized into 8 tiers (4 per tier):

| Tier | Focus          | Inventions                                                    |
|------|----------------|---------------------------------------------------------------|
| 1    | Decomposition  | LayerDecomposer, ConcernAnalyzer, BoundaryInferrer, CrossCuttingDetector |
| 2    | Entity         | EntityInferrer, RelationshipMapper, FieldDeriver, ValidationRuleGenerator |
| 3    | Operation      | OperationInferrer, SignatureGenerator, ErrorFlowDesigner, AsyncAnalyzer |
| 4    | Structure      | FileStructureGenerator, ImportGraphGenerator, ModuleHierarchyBuilder, ConfigDesigner |
| 5    | Dependency     | DependencyInferrer, VersionResolver, ApiSpecExtractor, ConflictResolver |
| 6    | Blueprint      | SkeletonGenerator, TypeFirstMaterializer, ContractSpecifier, GenerationPlanner |
| 7    | Integration    | WiringDiagramBuilder, DataFlowSpecifier, InitSequencer, ShutdownSequencer |
| 8    | Test           | TestCaseGenerator, TestFixtureDesigner, IntegrationTestPlanner, MockSpecifier |
