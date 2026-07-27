// Shared BlockCanvas no-code grammar — the ONE authoring surface reused by
// policy (Cedar P→R→A→Effect), workflow (Trigger→Condition→Branch→Action),
// Automate, and config rules.
//
// Mount, in short:
//   const [doc, setDoc] = useState(emptyDoc());        // or parseDoc(persisted)
//   <BlockCanvas doc={doc} strings={ko.console...canvas} onChange={setDoc} />
//   <PredicateEditor group={g} registry={FIELD_REGISTRY} strings={s} onChange={setG} />
//   <SimulationPanel group={g} registry={FIELD_REGISTRY} strings={s} samples={SAMPLES} />
// where FIELD_REGISTRY and SAMPLES come from the consumer's governed data
// contract. Test fixtures are deliberately not exported from this barrel.

export { BlockCanvas, type BlockCanvasProps } from "./BlockCanvas";
export { CanvasNodeCard, type CanvasNodeCardProps } from "./CanvasNodeCard";
export { PredicateEditor, type PredicateEditorProps } from "./PredicateEditor";
export { SimulationPanel, type SimulationPanelProps } from "./SimulationPanel";

export { DEFAULT_CANVAS_STRINGS, type CanvasStrings } from "./strings";

export {
  CANVAS_DOC_VERSION,
  connect,
  emptyDoc,
  moveNode,
  nodePorts,
  parseDoc,
  removeEdge,
  serializeDoc,
  upsertNode,
  validateDoc,
} from "./doc";

export {
  defaultOperatorForField,
  defaultValueForField,
  evalGroup,
  evalPredicate,
  runSimulation,
  type SimulationResult,
} from "./predicate";

export {
  CANVAS_NODE_KINDS,
  OPERATORS_BY_TYPE,
  OPERATOR_SYMBOL,
  type CanvasDoc,
  type CanvasEdge,
  type CanvasNode,
  type CanvasNodeKind,
  type CanvasOutput,
  type CanvasVar,
  type FieldChoice,
  type FieldDef,
  type FieldRegistry,
  type FieldType,
  type Predicate,
  type PredicateGroup,
  type PredicateOperator,
  type PredicateValue,
  type SampleRow,
} from "./types";
