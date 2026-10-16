O='vulcan-overview';S='vulcan-schema-reference';T='vulcan-tool-overview';W='vulcan-widget';A='announcements-release-notes'
specs=[
('F08.cad-model-scenes','Load multiple authorized CAD mesh drawing or point-cloud objects into one scene from stable media references and declared formats with explicit unsupported-input outcomes',O,'In the standalone application, select Add to scene, then choose a CAD object. Repeat this process to add more models to the scene.',['F04.media-format-ingestion','F04.media-reference-identity','F05.schema','F16.derived']),
('F08.cad-scene-navigation','Inspect assembled models unique-part grids and selected components with accessible scene trees camera controls and persistent authorized object context',W,'Grid: Display a sortable grid of unique parts.',['F08.cad-model-scenes','F10.saved-application-state']),
('F08.cad-annotations','Create typed annotations attached to stable part instances with validated 3D points labels intent and server-attributed authorship through the canonical action owner',O,'Annotate, measure, section, and recolor parts of a model.',['F08.cad-model-scenes','F06.typed','F06.action-recovery']),
('F08.cad-measurements','Measure ordered 3D polylines with explicit model axis and physical versus scene units, save reproducible measurement objects and expose typed results',T,'A measurement is an ordered set of points, and Vulcan reports the total distance along the polyline they define.',['F08.cad-model-scenes','F06.typed','F06.action-recovery']),
('F08.cad-section-planes','Inspect model cross-sections and save validated plane positions and quaternion orientations as governed objects that reopen with current authority',T,'A saved section plane stores its position and orientation so that the cross-section can be recalled later or shared across sessions and users.',['F08.cad-model-scenes','F06.typed','F06.action-recovery']),
('F08.cad-part-styling','Style meshes in normal X-Ray and wireframe modes and apply typed property/function driven part colors opacity and precedence without changing source geometry or disclosure policy',T,'You can switch between Normal, X-Ray, and Wireframe views.',['F08.cad-model-scenes','F11.function-dependency-resolution']),
('F08.cad-part-transforms','Apply validated finite column-major 4x4 transforms to stable part-instance or assembly identities from pinned authorized function outputs with explicit invalid-input outcomes',T,'Part transforms use function outputs to reposition parts or assemblies in space.',['F08.cad-model-scenes','F11.function-dependency-resolution']),
('F10.cad-widget-bindings','Embed governed CAD viewports in native applications with typed model scenario selection camera click and measurement bindings, canonical save actions and independently authorized tools',W,'Its inputs, outputs, and interface can also connect to Workshop variables so that Vulcan can interact with other widgets in the module.',['F08.cad-model-scenes','F08.cad-annotations','F08.cad-measurements','F08.cad-section-planes','F08.cad-part-styling','F08.cad-part-transforms','F10.typed-variable-bindings','F10.application-access-diagnostics'])
]
existing=[('F05.schema',S,'Vulcan looks up properties by their type classes, never by name, so you can name properties to fit your own Ontology conventions.'),('F06.typed',S,'Vulcan writes annotations, measurements, and section planes back to the Ontology through actions.'),('F04.media-format-ingestion',O,'Mesh formats: GLB, GLTF, STL, OBJ, PLY, 3MF'),('F08.cad-model-scenes',A,'Vulcan is available in beta across all enrollments starting the week of September 7.'),('F10.cad-widget-bindings',A,'camera position, part selection, and measurement results can be connected bidirectionally to Workshop variables.')]
qs=[
(O,'Vulcan is in the beta phase of development and may not be available on your enrollment. Functionality may change during active development.'),
(O,'A CAD object is an Ontology object with a property carrying the cad_media_reference type class, which holds a media reference to a model file.'),
(O,'Annotations, measurements, and section planes saved from Vulcan are Ontology objects, so they can drive other Palantir platform workflows such as alerts and dashboards.'),
(S,'Schema details may evolve as Vulcan develops. Verify your schema against this page after each Vulcan release.'),
(S,'All type classes listed below are in the vulcan namespace — that is, their kind field is always "vulcan" and only the name field varies.'),
(S,'Vulcan detects the format from the file extension.'),
(S,'Accepted values are X, Y, and Z. If omitted, Vulcan applies a format-specific default: Y for GLB and GLTF, Z for STL, OBJ, PLY, 3MF, DXF, LAS, and LAZ.'),
(S,'Accepted values are m (or meter / metre) and mm (or millimeter / millimetre). Vulcan ignores any other value.'),
(S,'This property has no effect on GLB, GLTF, or 3MF, because Vulcan always reads the unit those formats declare.'),
(S,'Measurement points are serialized as a flat array of numbers in the order [x1, y1, z1, x2, y2, z2, ...], expressed in the model\'s coordinate space.'),
(S,'Vulcan does not enforce semantic meaning for these values, so you can set them to match your workflow.'),
(S,'Set the creator property — annotation_created_by, measurement_created_by, or section_plane_created_by — to the user who submitted the action.'),
(S,'Vulcan supplies annotation_timestamp only for Create annotation.'),
(W,'Vulcan loads the first object in the set. To render multiple models, add a separate model entry for each object set.'),
(W,'A part ID selects every instance of that part in this model; a part instance ID selects one specific instance.'),
(W,'Scenario: The Workshop primary scenario to use when loading Ontology objects.'),
(W,'Exactly 16 finite numbers representing a 4×4 transformation matrix in column-major order.'),
(W,'The matrix values must use the order [m11, m21, m31, m41, m12, m22, m32, m42, m13, m23, m33, m43, m14, m24, m34, m44]. Vulcan ignores entries with a missing ID or an invalid matrix.'),
(W,'Each struct must define at least one of color, opacity, or selectedOpacity to affect the model.'),
(W,'If the Scene Coloring tool also assigns a color, the scene coloring value takes precedence while the configured opacity values remain in effect.'),
(W,'Camera target, position, and zoom bindings work in both directions.'),
(W,'Click X / Y / Z: Output number variables for the 3D coordinates of the point on a model part that the user most recently selected. Bind all three variables to reconstruct the selected point.'),
(W,'All options are on by default.'),
(W,'By default, Vulcan uses Complete mode, an Orthographic camera, and Orbit controls. Mode switching and part information are on; auto-rotate is off.'),
(W,'If you do not configure an action, Vulcan discovers actions that have the required parameter type classes and lets the user choose from the matches.'),
(W,'Configuring an action limits saving to that action and can provide defaults or user input fields for additional action parameters.'),
(W,'Measurement scene distance: The total measurement distance as a number in the model\'s scene units.'),
(W,'Measurement unit distance: The same distance as a formatted string in the user\'s selected display unit, such as 12.30 mm. If the model has no recognized unit, the output uses units.'),
(T,'When neither the file nor the CAD object supplies a unit, you can set one for the current session with Scene units in Measurement settings. This setting appears only in that case, and it resets when you reload the page.'),
(A,'Vulcan is available in beta across all enrollments starting the week of September 7.'),
(A,'When saved, annotations, measurements, and section planes created in Vulcan are written to the Ontology as objects.')
]
limits=[
'Vulcan is a bounded CAD/3D reference proposal, not exhaustive Foundry reconciliation or implementation evidence. The overview was captured before the September19 America/New_York cutoff; schema/tools/widget and release-note pages were captured after cutoff with exact timestamps preserved.',
'The post-cutoff release-notes JSON contains vendor announcement0c3d7d64-ff10-45ef-9a6e-7684478055b4 dated September8,2026 and beta availability starting weekSeptember7. This supports a dated public vendor assertion for broad formats/tools/widgets; it is not immutable historical page custody or proof that post-cutoff schema/tool details were identical on September19. Last-Modified alone is not historical proof.',
'Exact post-cutoff type classes, required fields, action parameter bindings, units/defaults, return schemas, precedence and navigation support retain historical-validity HOLD until source revision evidence or independent accepted freeze qualification is available. All newly required leaves remain planned and all executable bindings null.',
'Beta rollout wording differs: overview warns some enrollments may lack availability; dated announcement says all enrollments starting weekSeptember7. Record both vendor claims; no Console availability or support guarantee follows.',
'Nine directly rendered formats are GLB GLTF STL OBJ PLY 3MF DXF LAS LAZ. File extensions are vendor detection hints, not security validation. Parser fidelity, external references, decompression bounds, corrupted files, coordinate units, precision and resource saturation need isolated validated processing and independent fixtures.',
'Annotation measurement and section writes must use canonical authorized Actions with current field/source policy, immutable model/media versions, strict finite/shape/identity validation, expected revisions and command receipts. Creator identity is server-attributed; client schema IDs and hidden toolbar controls never grant authority.',
'Vendor ignores unknown model-unit values and invalid transform entries. Console must keep strict input validation and explicit unsupported/conflicting outcomes; silently copying vendor fallback behavior does not qualify correctness. Unit declarations and resets must be visible and reproducible when measurements are saved.',
'Property/function coloring and part transforms require pinned authorized inputs; styling must not expose restricted fields through colors measurements scene trees or selection state. Stored annotations planes and widget outputs inherit source restrictions and current revocation.',
'Widget scenario semantics and cross-resource branch consistency are not fully qualified by this bounded packet. Object-set first-item behavior, multi-model stable identity, two-way selection/camera and one-way click/measurement outputs require precise versioned contracts and recovery evidence.',
'Normal X-Ray wireframe colors and 3D controls require accessible alternatives, keyboard operation, Korean text, readable non-color intent, narrow/zoom navigation, cancellation and bounded GPU/memory envelopes. Screenshots in source docs are not Console browser acceptance.',
'No geometry manipulation scientific/engineering accuracy regulatory conclusion or physical unit calibration is established by these references; independent numerical and domain oracles remain required before workflow acceptance.'
]
