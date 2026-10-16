O='flow-capture-overview';R='flow-capture-record-a-workflow';G='flow-capture-generate-documentation'
specs=[
('F09.workflow-capture-resources','Create and reopen a named workflow capture with click screenshots and contextual metadata bound to authorized source resources',R,'Choose a file name and location to save your resource in Compass.',['F09.documents','F04.media-reference-identity','F16.derived']),
('F09.workflow-recorder-controls','Record same-origin application workflows with automatic or keyboard/manual screenshots and explicit frame dimensions; pause resume finish or discard with truthful durable outcomes',R,'Select Discard and exit to cancel the session and remove any captured media.',['F09.workflow-capture-resources','F10.mobile-application-interactions']),
('F09.capture-asset-editing','Stage screenshot crops and blur edits, import validated images, restore unsaved deletions and save versioned capture assets under explicit retention and legal-hold rules',R,'Deleted items remain recoverable until you save your changes by selecting Save in the edit toolbar.',['F09.workflow-capture-resources','F04.image-transform-library','F04.media-retention','F16.upload-execution-isolation']),
('F09.capture-audio-transcripts','Record optional microphone narration with explicit permission and source restrictions, associate timestamped audio transcripts with screenshots, and correct or regenerate transcript versions through isolated processing',R,'If recorded, your audio will be transcribed automatically and associated with the captured screenshots.',['F09.workflow-capture-resources','F04.audio-transform-library','F16.derived']),
('F09.capture-document-export','Preview manually authored capture documentation and export governed document, ordered walkthrough, PDF and ZIP Markdown/image versions with explicit conversion differences and current disclosure checks',G,'Export as .zip: Bundles Markdown content and images into a .zip file that can be downloaded.',['F09.workflow-capture-resources','F09.document-export','F09.document-markdown-conversion','F16.download-governance']),
('F16.capture-source-policy','Inherit captured source and field restrictions across screenshots audio transcripts manual documentation copies and exports, enforce current revocation, and require explicit release authority for broader disclosure',O,'It is the user’s responsibility to ensure that appropriate security checks related to markings and group-based access are applied to created Flow Capture resources to reflect the security checks of the captured content.',['F16.derived','F16.sharing','F16.download-governance','F16.sensitive-action-checkpoints','F09.workflow-capture-resources'])
]
existing=[('F09.documents',O,'View mode displays generated documentation in presentation or preview form, while edit mode can be used to modify generated Markdown, create additional recordings, and adjust images or transcriptions before re-generating, saving, or exporting.'),('F16.sensitive-action-checkpoints',O,'You can configure a checkpoint to warn users before they perform sensitive actions, such as exporting generated content.'),('F16.image-obfuscation',O,'Crop and blur areas of captured images directly in Flow Capture to remove or redact sensitive areas before generation or export.')]
qs=[
(O,'Flow Capture is in the beta phase of development and may not be available on your enrollment. Functionality may change during active development.'),
(O,'To enable Flow Capture, contact your platform administrator to modify application access in Control Panel.'),
(O,'It records mouse clicks, screenshots, optional audio with transcription, and contextual metadata while you navigate an application in a recording session.'),
(O,'Screenshots are stored as individual snapshot assets and can be included in the final documentation in context, or as standalone images.'),
(O,'Audio files and their transcriptions are stored as assets and can be used to enrich documentation generation by providing additional context and instructions.'),
(R,'Before recording, ensure that you understand your organization\'s data handling policies.'),
(R,'In the acknowledgement dialog, select the checkbox to confirm the required security settings, and then select Acknowledge.'),
(R,'The URL field only accepts same-domain URLs. The page you navigate to must be on the same Foundry enrollment that Flow Capture is running on.'),
(R,'If Auto-screenshot is enabled, screenshots will be taken automatically for every click event, but you may take manual screenshots at any time.'),
(R,'Use the Take screenshot option in the recorder controls to capture the current view, or use the keyboard shortcut Cmd+Shift+S (macOS) or Ctrl+Shift+S (Windows) to capture a screenshot.'),
(R,'Full size: Capture at the full resolution of the target page.'),
(R,'Scaled: Scale the page to fit the recorder viewport.'),
(R,'Custom: Specify custom dimensions for the capture frame.'),
(R,'Select Pause to pause the capture.'),
(R,'Select Record to resume after pausing.'),
(R,'When you have finished capturing the workflow, select Done.'),
(R,'Edits are staged until you save them.'),
(R,'You can also import images into Flow Capture by selecting Upload in the top right corner of the Assets page, and selecting the files you want to upload.'),
(R,'After selecting Save, deleted images and audio will be permanently removed.'),
(R,'To regenerate the transcription automatically, select Regenerate.'),
(G,'Export as Walkthrough: Converts the content to a Walkthrough. Walkthroughs enforce a step-by-step structure, so Walkthrough content may differ after export from Flow Capture.'),
(G,'Export as Notepad: Converts the content to a Notepad document.'),
(G,'Export as PDF: Renders Markdown content as a PDF document that can be downloaded.'),
(G,'After content has been generated, you can edit the results by switching to the Documentation tab and selecting Edit in the top right corner. Content will be displayed using Markdown syntax.'),
(G,'Only assets that you have added to context are visible to the model during generation.'),
(G,'Different models support different images quantities per request.'),
(O,'Choose a system prompt or template, and a model to influence how content is generated.'),
(G,'Regenerate documentation: Overwrite existing documentation with a new prompt.'),
(G,'Append to or modify current documentation: Edit the existing documentation using the prompt.')
]
limits=[
'Flow Capture is a bounded three-page reference capture, not exhaustive Foundry reconciliation or proof of Console implementation. Beta/administrator enablement does not remove requested classical capabilities.',
'Flow Capture classical scope includes capture resources, recorder controls, screenshots, optional audio/transcripts, asset edits, manual Markdown preview and governed export. Prompt templates/model selection, LLM context inclusion, LLM generation, regeneration and prompt-driven append/modify are classified on the separate Intelligence roadmap; this does not exclude the whole workflow application.',
'Optional transcription is required through the isolated classical audio processing contract; provider/model/runtime support and confidentiality must be independently qualified. No microphone permission, transcription quality, timestamp fidelity, browser capture engine or supported browser/version has been established by these public references.',
'Vendor documentation places responsibility for matching markings/group access on users. Console instead requires source/field restriction inheritance and current-authority checks for every capture artifact, transcript, document, copy, preview and export. A warning checkpoint or same-domain recorder is not authorization; blur is not proof of irreversible redaction or a disclosure release.',
'Vendor saved asset deletion is documented as permanent. Console must provide an explicit retention/legal-hold compatibility contract and must not copy this behavior in violation of required historical evidence or recovery. Actual physical deletion and derived-copy cleanup remain independently qualified.',
'Walkthrough export may alter content to fit ordered steps. Required native successor equivalence must retain stable references, disclose conversion loss, provide accessible navigation and preserve source policy. No requirement to reproduce vendor application names or use vendor APIs is implied.',
'Recorded clicks and URLs are contextual evidence only; recorder replay must never execute consequential business commands without the canonical owner and current authority. Capture must not expose cookies, tokens, confidential hidden fields or unrelated working context.',
'All new leaves remain planned with null executable bindings. Capture correctness, policy revocation, hostile uploads, Korean IME/keyboard/320px/zoom, interrupted save/export, restoration, resource limits and independent browser acceptance remain release blockers.'
]
