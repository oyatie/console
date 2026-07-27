import Foundation
import ConsoleAPIClient
import ConsoleCore
import SwiftUI

#if os(iOS)
import UIKit
#endif

struct ConsoleRootView: View {
    @StateObject var viewModel: ConsoleViewModel

    var body: some View {
        Group {
            if viewModel.isAuthenticated {
                ConsoleAuthenticatedTabs(viewModel: viewModel)
            } else {
                NavigationStack {
                    LoginView(viewModel: viewModel)
                        .navigationTitle(Text("app_name"))
                        .inlineNavigationTitle()
                }
            }
        }
        .task {
            viewModel.restore()
        }
        .sheet(isPresented: $viewModel.isCameraPresented) {
            CameraCaptureView { url in
                Task {
                    await viewModel.evidenceCaptured(fileURL: url)
                    viewModel.isCameraPresented = false
                }
            } onCancel: {
                viewModel.isCameraPresented = false
            } onError: {
                viewModel.cameraCaptureFailed()
                viewModel.isCameraPresented = false
            }
        }
    }
}

struct ConsoleAuthenticatedTabs: View {
    @ObservedObject var viewModel: ConsoleViewModel

    var body: some View {
        TabView {
            UnobscuredTabContent {
                NavigationStack {
                    TodayListView(viewModel: viewModel)
                }
            }
                .tabItem {
                    Label("today_title", systemImage: "list.bullet")
                }
            UnobscuredTabContent {
                NavigationStack {
                    WorkHubTabView(viewModel: viewModel)
                        .accessibilityIdentifier(ConsoleAccessibilityID.workHubTab)
                }
            }
                .tabItem {
                    Label("work_hub_title", systemImage: "square.grid.2x2")
                }
            UnobscuredTabContent {
                NavigationStack {
                    // The identifier is attached to the List inside
                    // MessengerTabView, whose body root is now a VStack; the
                    // UI tests query it as a collection view.
                    MessengerTabView(viewModel: viewModel)
                }
            }
                .tabItem {
                    Label("messenger_title", systemImage: "message.fill")
                }
            UnobscuredTabContent {
                NavigationStack {
                    OperationsTabView(viewModel: viewModel)
                        .accessibilityIdentifier(ConsoleAccessibilityID.operationsTab)
                }
            }
                .tabItem {
                    Label("operations_title", systemImage: "tray.full")
                }
        }
        .accessibilityIdentifier(ConsoleAccessibilityID.authenticatedTabs)
    }
}

private struct UnobscuredTabContent<Content: View>: View {
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    @ViewBuilder
    var body: some View {
        #if os(iOS)
        if #available(iOS 26.0, *) {
            TabBarContentLayoutGuideHost(content: content)
        } else {
            content
        }
        #else
        content
        #endif
    }
}

#if os(iOS)
@available(iOS 26.0, *)
@MainActor
private struct TabBarContentLayoutGuideHost<Content: View>: UIViewControllerRepresentable {
    let content: Content

    func makeUIViewController(context: Context) -> TabBarContentLayoutGuideHostController<Content> {
        TabBarContentLayoutGuideHostController(content: content)
    }

    func updateUIViewController(
        _ uiViewController: TabBarContentLayoutGuideHostController<Content>,
        context: Context
    ) {
        uiViewController.update(content: content)
    }

    static func dismantleUIViewController(
        _ uiViewController: TabBarContentLayoutGuideHostController<Content>,
        coordinator: ()
    ) {
        uiViewController.invalidate()
    }
}

@available(iOS 26.0, *)
@MainActor
private final class TabBarContentLayoutGuideContainerView: UIView {
    var onHierarchyChange: (@MainActor () -> Void)?

    override func didMoveToWindow() {
        super.didMoveToWindow()
        onHierarchyChange?()
    }
}

@available(iOS 26.0, *)
@MainActor
private final class TabBarContentLayoutGuideHostController<Content: View>: UIViewController {
    private let hostingController: UIHostingController<Content>
    private weak var observedTabBarController: UITabBarController?
    private var contentLayoutConstraints: [NSLayoutConstraint] = []

    init(content: Content) {
        hostingController = UIHostingController(rootView: content)
        super.init(nibName: nil, bundle: nil)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func loadView() {
        let containerView = TabBarContentLayoutGuideContainerView()
        containerView.backgroundColor = .clear
        containerView.onHierarchyChange = { [weak self] in
            self?.constrainHostedContentToTabLayoutGuide()
        }
        view = containerView
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        addChild(hostingController)
        hostingController.view.translatesAutoresizingMaskIntoConstraints = false
        hostingController.view.backgroundColor = .clear
        view.addSubview(hostingController.view)
        hostingController.didMove(toParent: self)
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        constrainHostedContentToTabLayoutGuide()
    }

    func update(content: Content) {
        hostingController.rootView = content
        constrainHostedContentToTabLayoutGuide()
    }

    private func constrainHostedContentToTabLayoutGuide() {
        guard let tabBarController else {
            NSLayoutConstraint.deactivate(contentLayoutConstraints)
            contentLayoutConstraints = []
            observedTabBarController = nil
            return
        }
        guard
            let window = viewIfLoaded?.window,
            tabBarController.viewIfLoaded?.window === window,
            hostingController.view.isDescendant(of: tabBarController.view)
        else {
            NSLayoutConstraint.deactivate(contentLayoutConstraints)
            contentLayoutConstraints = []
            observedTabBarController = nil
            return
        }
        guard observedTabBarController !== tabBarController || contentLayoutConstraints.isEmpty else {
            return
        }

        NSLayoutConstraint.deactivate(contentLayoutConstraints)
        let guide = tabBarController.contentLayoutGuide
        contentLayoutConstraints = [
            hostingController.view.topAnchor.constraint(equalTo: guide.topAnchor),
            hostingController.view.leadingAnchor.constraint(equalTo: guide.leadingAnchor),
            hostingController.view.bottomAnchor.constraint(equalTo: guide.bottomAnchor),
            hostingController.view.trailingAnchor.constraint(equalTo: guide.trailingAnchor),
        ]
        NSLayoutConstraint.activate(contentLayoutConstraints)
        observedTabBarController = tabBarController
    }

    func invalidate() {
        NSLayoutConstraint.deactivate(contentLayoutConstraints)
        contentLayoutConstraints = []
        observedTabBarController = nil
        guard hostingController.parent === self else { return }
        hostingController.willMove(toParent: nil)
        hostingController.view.removeFromSuperview()
        hostingController.removeFromParent()
    }
}
#endif


struct WorkHubTabView: View {
    @ObservedObject var viewModel: ConsoleViewModel

    var body: some View {
        let summary = viewModel.workHubSummary
        List {
            Section {
                Text(workHubCountText("work_hub_today_count_format", summary.todayWorkCount))
                Text(workHubCountText("work_hub_urgent_count_format", summary.urgentWorkCount))
                Text(workHubCountText("work_hub_target_due_count_format", summary.targetDueWorkCount))
                LabeledContent("location_consent_collection", value: localizedString(summary.gpsMayCollect ? "yes" : "no"))
            } header: {
                Text("work_hub_daily_section")
            } footer: {
                Text("work_hub_daily_footer")
            }

            if let messageKey = viewModel.messageKey {
                Section {
                    Text(LocalizedStringKey(messageKey))
                }
            }

            Section {
                Text(workHubCountText("work_hub_approval_count_format", summary.approvalRelatedCount))
                Text(workHubCountText("work_hub_pending_sync_format", summary.pendingSyncCount))
                Label("work_hub_passkey_required", systemImage: "person.badge.key")
                Text("work_hub_sensitive_note")
            } header: {
                Text("work_hub_sensitive_section")
            }

            Section {
                ForEach(summary.collaborationActions) { action in
                    WorkHubActionRow(action: action)
                        .accessibilityIdentifier(ConsoleAccessibilityID.workHubCollaborationAction(action.kind.rawValue))
                }
            } header: {
                Text("work_hub_collaboration_section")
            } footer: {
                Text("work_hub_collaboration_footer")
            }
        }
        .accessibilityIdentifier(ConsoleAccessibilityID.workHubList)
        .navigationTitle(Text("work_hub_title"))
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    Task {
                        await viewModel.refreshWorkHub()
                    }
                } label: {
                    Label("refresh", systemImage: "arrow.clockwise")
                }
                Button {
                    Task { await viewModel.logout() }
                } label: {
                    Label("logout", systemImage: "rectangle.portrait.and.arrow.right")
                }
            }
        }
    }
}

private func workHubCountText(_ key: String, _ count: Int) -> String {
    String.localizedStringWithFormat(NSLocalizedString(key, comment: ""), count)
}

struct WorkHubActionRow: View {
    let action: MobileCollaborationAction

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(LocalizedStringKey(action.titleKey))
                    .font(.headline)
                Spacer()
                ConsoleChip(key: action.status.fieldLabelKey)
            }
            Text(workHubActionValue(action))
                .font(.subheadline)
            Text(LocalizedStringKey(action.detailKey))
                .font(.footnote)
                .foregroundStyle(.secondary)
            if action.requiresPasskey {
                Label("work_hub_action_passkey_step_up", systemImage: "person.badge.key")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

private func workHubActionValue(_ action: MobileCollaborationAction) -> String {
    if let count = action.count {
        return localizedString(action.valueKey, count)
    }
    return localizedString(action.valueKey)
}



struct OperationsTabView: View {
    @ObservedObject var viewModel: ConsoleViewModel

    var body: some View {
        let dashboard = viewModel.mobileOperationsDashboard
        List {
            if let overview = viewModel.mobileOperationsOverview, overview.origin == .cachedAfterFailure {
                Section {
                    Label("operations_cached_fallback", systemImage: "wifi.slash")
                }
            }

            Section {
                LabeledContent("operations_notification_badge", value: localizedString("operations_count_format", viewModel.mobileNotificationInbox.badgeCount))
                LabeledContent("operations_notification_unread", value: localizedString("operations_count_format", viewModel.mobileNotificationInbox.unreadCount))
                LabeledContent("operations_notification_urgent", value: localizedString("operations_count_format", viewModel.mobileNotificationInbox.urgentUnreadCount))
                ForEach(viewModel.mobileNotificationInbox.notifications.prefix(3)) { notification in
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(notification.title)
                                .font(.headline)
                            Spacer()
                            ConsoleChip(key: notification.isUrgent ? "operations_notification_urgent_chip" : "operations_notification_normal_chip")
                        }
                        Text(notification.body)
                            .font(.subheadline)
                        Text(notification.receivedAt.formatted(date: .abbreviated, time: .shortened))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Button("operations_notification_mark_read") {
                            Task { await viewModel.markNotificationRead(notification) }
                        }
                        .disabled(notification.isUnread == false)
                    }
                    .padding(.vertical, 4)
                }
            } header: {
                Text("operations_notification_section")
            }

            Section {
                LabeledContent("operations_approval_count", value: localizedString("operations_count_format", dashboard?.approvalCount ?? 0))
                if let approvals = dashboard?.approvals, approvals.isEmpty == false {
                    ForEach(approvals.prefix(3)) { approval in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(approval.title)
                                .font(.headline)
                            Text(approval.summary)
                                .font(.subheadline)
                            if approval.canExecuteOnMobile == false {
                                Text("operations_approval_unsupported_mobile")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                } else {
                    Text("operations_approval_empty")
                        .foregroundStyle(.secondary)
                }
                TextField(String(localized: "operations_approval_comment"), text: $viewModel.approvalComment, axis: .vertical)
                    .lineLimit(2...4)
                    .accessibilityIdentifier(ConsoleAccessibilityID.operationsApprovalCommentField)
                Button {
                    Task { await viewModel.queueFirstApprovalForPasskey() }
                } label: {
                    Label("operations_queue_approval", systemImage: "person.badge.key")
                }
                Button {
                    Task { await viewModel.replayMobileSensitiveActions() }
                } label: {
                    Label("operations_replay_sensitive_actions", systemImage: "arrow.triangle.2.circlepath")
                }
            } header: {
                Text("operations_approval_section")
            }

            Section {
                LabeledContent("operations_sensitive_waiting_passkey", value: localizedString("operations_count_format", viewModel.mobileSensitiveActionSummary.pendingPasskeyCount))
                LabeledContent("operations_sensitive_ready_replay", value: localizedString("operations_count_format", viewModel.mobileSensitiveActionSummary.readyForReplayCount))
                LabeledContent("operations_sensitive_failed", value: localizedString("operations_count_format", viewModel.mobileSensitiveActionSummary.failedCount))
            } header: {
                Text("operations_sensitive_section")
            }

            Section {
                LabeledContent("operations_mail_unread", value: localizedString("operations_count_format", dashboard?.unreadMailCount ?? 0))
                if let mailThreads = dashboard?.mailThreads, mailThreads.isEmpty == false {
                    ForEach(mailThreads) { thread in
                        OperationsMailThreadRow(thread: thread) {
                            Task { await viewModel.markMailThreadRead(thread) }
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.operationsMailThread(thread.id))
                    }
                } else {
                    Text("operations_mail_empty")
                        .foregroundStyle(.secondary)
                }
            } header: {
                Text("operations_mail_section")
            }

            Section {
                if let events = dashboard?.calendarEvents, events.isEmpty == false {
                    ForEach(events) { event in
                        OperationsCalendarEventRow(event: event)
                            .accessibilityIdentifier(ConsoleAccessibilityID.operationsCalendarEvent(event.id))
                    }
                } else {
                    Text("operations_calendar_empty")
                        .foregroundStyle(.secondary)
                }
            } header: {
                Text("operations_calendar_section")
            }

            Section {
                if let polls = dashboard?.polls, polls.isEmpty == false {
                    ForEach(polls) { poll in
                        OperationsPollRow(poll: poll) {
                            Task { await viewModel.votePoll(poll) }
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.operationsPoll(poll.id))
                    }
                } else {
                    Text("operations_poll_empty")
                        .foregroundStyle(.secondary)
                }
            } header: {
                Text("operations_poll_section")
            }

            if let messageKey = viewModel.messageKey {
                Section {
                    Text(LocalizedStringKey(messageKey))
                }
            }
        }
        .accessibilityIdentifier(ConsoleAccessibilityID.operationsList)
        .navigationTitle(Text("operations_title"))
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    Task { await viewModel.refreshMobileOperations() }
                } label: {
                    Label("refresh", systemImage: "arrow.clockwise")
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.operationsRefreshButton)
                Button {
                    Task { await viewModel.logout() }
                } label: {
                    Label("logout", systemImage: "rectangle.portrait.and.arrow.right")
                }
            }
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView("loading")
            }
        }
        .task {
            if viewModel.mobileOperationsOverview == nil {
                await viewModel.refreshMobileOperations()
            }
        }
    }
}

struct OperationsMailThreadRow: View {
    let thread: MobileMailThreadRow
    let onMarkRead: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(thread.subject)
                    .font(.headline)
                Spacer()
                if thread.unreadCount > 0 {
                    ConsoleChip(key: "operations_unread_chip")
                }
            }
            Text(thread.lastMessageAt.formatted(date: .abbreviated, time: .shortened))
                .font(.footnote)
                .foregroundStyle(.secondary)
            HStack {
                Text(localizedString("operations_mail_unread_format", thread.unreadCount))
                if thread.hasAttachments { ConsoleChip(key: "operations_attachment_chip") }
                if thread.isFlagged { ConsoleChip(key: "operations_flagged_chip") }
            }
            .font(.caption)
            Button("operations_mark_read", action: onMarkRead)
                .disabled(thread.unreadCount == 0)
        }
        .padding(.vertical, 4)
    }
}

struct OperationsCalendarEventRow: View {
    let event: MobileCalendarEventRow

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(event.title)
                    .font(.headline)
                Spacer()
                ConsoleChip(key: operationsScopeKey(event.scopeType))
            }
            Text(event.description)
                .font(.subheadline)
            Text(localizedString("operations_calendar_time_format", event.startsAt.formatted(date: .abbreviated, time: .shortened), event.endsAt.formatted(date: .omitted, time: .shortened)))
                .font(.caption)
                .foregroundStyle(.secondary)
            if let objectType = event.objectType {
                Text(localizedString("operations_object_link_format", objectType))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            if event.isCancelled {
                ConsoleChip(key: "operations_calendar_cancelled")
            }
        }
        .padding(.vertical, 4)
    }
}

struct OperationsPollRow: View {
    let poll: MobilePollRow
    let onVote: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(poll.title)
                    .font(.headline)
                Spacer()
                ConsoleChip(key: operationsPollStatusKey(poll.status))
            }
            Text(poll.question)
            HStack {
                ConsoleChip(key: operationsPollAnonymityKey(poll.anonymity))
                Text(localizedString("operations_poll_vote_count_format", poll.voteCount))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            if let option = poll.firstOptionLabel {
                Button {
                    onVote()
                } label: {
                    Label(localizedString("operations_poll_vote_option_format", option), systemImage: "checkmark.circle")
                }
                .disabled(poll.canVote == false)
            }
            if poll.hasSubmittedVote {
                Text("operations_poll_submitted")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

private func operationsScopeKey(_ scope: Components.Schemas.CollaborationScopeType) -> String {
    switch scope {
    case .tenant: "operations_scope_tenant"
    case .org: "operations_scope_org"
    case .department: "operations_scope_department"
    case .team: "operations_scope_team"
    case .personal: "operations_scope_personal"
    }
}

private func operationsPollStatusKey(_ status: Components.Schemas.PollStatus) -> String {
    switch status {
    case .draft: "operations_poll_status_draft"
    case .open: "operations_poll_status_open"
    case .closed: "operations_poll_status_closed"
    case .archived: "operations_poll_status_archived"
    }
}

private func operationsPollAnonymityKey(_ anonymity: Components.Schemas.PollAnonymity) -> String {
    switch anonymity {
    case .named: "operations_poll_named"
    case .anonymous: "operations_poll_anonymous"
    }
}

struct LoginView: View {
    @ObservedObject var viewModel: ConsoleViewModel

    var body: some View {
        Form {
            Section {
                TextField(String(localized: "user_id"), text: $viewModel.userID)
                    .autocorrectionDisabled()
                    #if os(iOS)
                    .textInputAutocapitalization(.never)
                    #endif
                    .accessibilityIdentifier(ConsoleAccessibilityID.loginUserIDField)
                Button {
                    Task { await viewModel.login() }
                } label: {
                    Text("login_button")
                        .font(.body)
                        .fixedSize(horizontal: false, vertical: true)
                        .frame(minHeight: 44)
                }
                .buttonStyle(.borderedProminent)
                .tint(.primary)
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(ConsoleAccessibilityID.loginButton)
            } header: {
                Text("login_title")
                    .foregroundStyle(.primary)
            }
            .headerProminence(.increased)

            if let messageKey = viewModel.messageKey {
                Text(LocalizedStringKey(messageKey))
                    .foregroundStyle(.red)
                    .accessibilityIdentifier(ConsoleAccessibilityID.loginErrorMessage)
            }
        }
    }
}

struct TodayListView: View {
    @ObservedObject var viewModel: ConsoleViewModel
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize
    @State private var isLocationConsentPresented = false

    var body: some View {
        List {
            if dynamicTypeSize.isAccessibilitySize == false {
                LocationConsentSection(viewModel: viewModel)
            }
            if let messageKey = viewModel.messageKey {
                Text(LocalizedStringKey(messageKey))
            }
            if viewModel.today.isEmpty {
                Text("empty_today")
                    .accessibilityIdentifier(ConsoleAccessibilityID.todayEmpty)
            }
            ForEach(viewModel.today) { workOrder in
                Button {
                    viewModel.select(workOrder)
                } label: {
                    WorkOrderRow(workOrder: workOrder)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(ConsoleAccessibilityID.workOrderRow(workOrder.id))
            }
        }
        .accessibilityIdentifier(ConsoleAccessibilityID.todayList)
        .navigationTitle(Text("today_title"))
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                if dynamicTypeSize.isAccessibilitySize {
                    Button {
                        isLocationConsentPresented = true
                    } label: {
                        Label("location_consent_title", systemImage: "location.circle")
                    }
                    .accessibilityIdentifier(ConsoleAccessibilityID.todayLocationConsentButton)
                }
                Button {
                    Task { await viewModel.refreshToday() }
                } label: {
                    Label("refresh", systemImage: "arrow.clockwise")
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.todayRefreshButton)
                Button {
                    Task { await viewModel.logout() }
                } label: {
                    Label("logout", systemImage: "rectangle.portrait.and.arrow.right")
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.todayLogoutButton)
            }
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView("loading")
                    .accessibilityIdentifier(ConsoleAccessibilityID.todayLoading)
            }
        }
        .sheet(isPresented: $isLocationConsentPresented) {
            NavigationStack {
                Form {
                    LocationConsentSection(viewModel: viewModel)
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.todayLocationConsentSheet)
                .navigationTitle(Text("location_consent_title"))
                .inlineNavigationTitle()
                .toolbar {
                    Button {
                        isLocationConsentPresented = false
                    } label: {
                        Label("close", systemImage: "xmark")
                    }
                    .accessibilityIdentifier(ConsoleAccessibilityID.todayLocationConsentCloseButton)
                }
            }
        }
        .workOrderDetailPresentation(item: $viewModel.selectedWorkOrder) { _ in
            WorkOrderDetailView(viewModel: viewModel)
        }
    }

}

struct MessengerTabView: View {
    @ObservedObject var viewModel: ConsoleViewModel
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize

    var body: some View {
        // The composer/send bar is a sibling of the List, never a row inside
        // it: the UIKit content-layout-guide host already excludes the tab bar
        // from this content area, so the bottom child sits above it without an
        // ad-hoc safe-area inset (which the tab-host seam deliberately forbids).
        VStack(spacing: 0) {
        List {
            Section {
                HStack {
                    ZStack(alignment: .leading) {
                        if viewModel.messengerSearchQuery.isEmpty {
                            Text("messenger_search")
                                .foregroundStyle(.primary)
                                .accessibilityHidden(true)
                        }
                        TextField("", text: $viewModel.messengerSearchQuery, axis: .vertical)
                            .accessibilityLabel(Text("messenger_search"))
                            .accessibilityIdentifier(ConsoleAccessibilityID.messengerSearchField)
                    }
                        .lineLimit(1...2)
                        .layoutPriority(1)
                    Button {
                        Task { await viewModel.searchMessengerMessages() }
                    } label: {
                        Label("messenger_search_button", systemImage: "magnifyingglass")
                            .labelStyle(.iconOnly)
                            .foregroundStyle(.primary)
                            .frame(minWidth: 44, minHeight: 44)
                            .contentShape(Rectangle())
                    }
                    .accessibilityLabel(Text("messenger_search_button"))
                    .accessibilityIdentifier(ConsoleAccessibilityID.messengerSearchButton)
                    .buttonStyle(.plain)
                    .tint(.primary)
                }
                if viewModel.messengerState.searchResults.isEmpty == false {
                    ForEach(viewModel.messengerState.searchResults) { message in
                        MessengerMessageRow(
                            message: message,
                            currentUserID: viewModel.currentUserID,
                            accessibilityIdentifier: ConsoleAccessibilityID.messengerSearchResultRow(message.id)
                        )
                    }
                } else if viewModel.messengerHasSearched {
                    Text("messenger_search_no_results")
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier(ConsoleAccessibilityID.messengerSearchNoResults)
                }
            }

            Section {
                Text("messenger_threads")
                    .font(.headline)
                    .fixedSize(horizontal: false, vertical: true)
                    .foregroundStyle(.primary)
                    .accessibilityAddTraits(.isHeader)
                    .listRowBackground(Color.clear)
                    .listRowSeparator(.hidden)

                if viewModel.messengerState.threads.isEmpty {
                    Text("messenger_empty_threads")
                        .accessibilityIdentifier(ConsoleAccessibilityID.messengerEmptyThreads)
                }
                ForEach(viewModel.messengerState.threads) { thread in
                    Button {
                        Task { await viewModel.selectMessengerThread(thread) }
                    } label: {
                        MessengerThreadRow(
                            thread: thread,
                            isSelected: viewModel.messengerState.selectedThreadID == thread.id
                        )
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .accessibilityIdentifier(ConsoleAccessibilityID.messengerThreadRow(thread.id))
                    // Selection is a trait, not a caption. A visible "selected"
                    // label duplicated state VoiceOver already conveys, and its
                    // frame settled under the opaque navigation bar at AX5, where
                    // the audit measured it at 1.03:1 against the bar's own fill.
                    .accessibilityAddTraits(
                        viewModel.messengerState.selectedThreadID == thread.id ? [.isSelected] : []
                    )
                }
            }

            Section {
                Text("messenger_messages")
                    .font(.headline)
                    .fixedSize(horizontal: false, vertical: true)
                    .foregroundStyle(.primary)
                    .accessibilityAddTraits(.isHeader)
                    .listRowBackground(Color.clear)
                    .listRowSeparator(.hidden)

                if let selectedThreadID = viewModel.messengerState.selectedThreadID {
                    let messages = viewModel.messengerState.messagesByThread[selectedThreadID] ?? []
                    // Dictionary lookup returns a nested optional because the
                    // stored cursor is itself optional. A present key with a
                    // nil cursor means there is no older page to load.
                    if let _ = viewModel.messengerState.nextCursorByThread[selectedThreadID] ?? nil {
                        Button {
                            Task { await viewModel.loadOlderMessengerMessages() }
                        } label: {
                            HStack(alignment: .firstTextBaseline, spacing: 8) {
                                Image(systemName: "arrow.up.circle")
                                    .accessibilityHidden(true)
                                Text("messenger_load_older")
                                    .font(.body)
                                    .multilineTextAlignment(.leading)
                            }
                            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
                        }
                    }
                    if messages.isEmpty {
                        Text("messenger_empty_messages")
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    ForEach(messages) { message in
                        MessengerMessageRow(
                            message: message,
                            currentUserID: viewModel.currentUserID,
                            accessibilityIdentifier: ConsoleAccessibilityID.messengerMessageRow(message.id)
                        )
                    }
                    // The composer and send action deliberately do NOT live here.
                    // Inside this lazy List they were the last rows of the
                    // message section, so a long thread scrolled the primary
                    // message action off-screen and it was never materialized
                    // for accessibility queries. They are now a persistent
                    // sibling of this List; see `messengerComposerBar`.
                } else {
                    Text("messenger_select_thread")
                        .accessibilityIdentifier(ConsoleAccessibilityID.messengerSelectThreadPrompt)
                }
            }

            if let messageKey = viewModel.messageKey {
                Text(LocalizedStringKey(messageKey))
            }
        }
        // These stay attached to the List, not the enclosing VStack. The tab
        // identifier must land on the collection view the UI tests query, and
        // `safeAreaInset(edge: .top)` only reserves a scroll-safe viewport when
        // it insets a scroll view — on a VStack it degrades to ordinary layout
        // padding above the whole stack, which looks identical at rest.
        .accessibilityIdentifier(ConsoleAccessibilityID.messengerTab)
        .navigationTitle(Text("messenger_title"))
        // Messenger contains long, changing operational conversations. Keep the
        // navigation title and actions on an opaque semantic surface rather
        // than allowing scrolling content to alter their contrast.
        .messengerNavigationBarBackground()
        // iOS 26 can otherwise settle a selected AX5 List row underneath the
        // persistent navigation surface. Reserve a real scroll-safe viewport;
        // this preserves the complete thread rather than hiding or truncating it.
        .safeAreaInset(edge: .top, spacing: 0) {
            if dynamicTypeSize == .accessibility5 {
                Color.clear
                    .frame(height: 56)
                    .accessibilityHidden(true)
            }
        }
            if viewModel.messengerState.selectedThreadID != nil {
                messengerComposerBar
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    Task { await viewModel.refreshMessenger() }
                } label: {
                    Label("refresh", systemImage: "arrow.clockwise")
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.messengerRefreshButton)
                Button {
                    Task { await viewModel.logout() }
                } label: {
                    Label("logout", systemImage: "rectangle.portrait.and.arrow.right")
                }
                .accessibilityIdentifier(ConsoleAccessibilityID.messengerLogoutButton)
            }
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView("loading")
            }
        }
        .task {
            if viewModel.messengerState.threads.isEmpty {
                await viewModel.refreshMessenger()
            }
        }
    }

    /// Persistent composer and send action, presented as a sibling of the List
    /// inside a `VStack` rather than as trailing rows of the lazy message List.
    ///
    /// Deliberately NOT a bottom safe-area inset: `hasUnobscuredTabContentHost`
    /// forbids that construct anywhere in this file — it scans the raw source,
    /// so even naming it here trips the gate — because the UIKit
    /// `contentLayoutGuide` tab-host seam owns bottom insets. Do not "fix" this
    /// to match a mental model of a bottom inset.
    ///
    /// Bindings, send semantics, accessibility identifiers, the contrast-safe
    /// placeholder and its `allowsHitTesting(false)` are carried over verbatim
    /// from the previous in-List rows; only the placement changed.
    ///
    /// The composer's line limit tightens at accessibility sizes. Threads
    /// auto-select on load — `MessengerState` falls back to `threads.first` —
    /// so this bar is on screen from the moment Messenger appears, not only
    /// once someone opens a conversation. At AX5 a five-line composer is tall
    /// enough to push the first thread row's kind chip outside the List's own
    /// frame, which is a real loss of conversation content rather than merely
    /// a failing assertion. Two lines keeps the composer usable while leaving
    /// the thread list legible.
    ///
    /// Keep prose here rather than inside the body: `opaqueUnobscuredSurface`
    /// caps the span from `Divider()` to the `.background` at 2,200 characters
    /// and counts comments, so body comments have a hard length budget.
    private var messengerComposerBar: some View {
        VStack(spacing: 0) {
            // Opaque semantic surface: scrolling message content must never
            // alter the contrast of the primary action, and the floating tab
            // bar must not obscure it.
            Divider()
            HStack(alignment: .bottom) {
                // Native TextField prompts retain placeholder opacity
                // even with a primary foreground, which falls below
                // contrast requirements at accessibility text sizes.
                ZStack(alignment: .leading) {
                    if viewModel.messengerDraft.isEmpty {
                        Text("messenger_composer")
                            .foregroundStyle(.primary)
                            .accessibilityHidden(true)
                            .allowsHitTesting(false)
                    }
                    TextField("", text: $viewModel.messengerDraft, axis: .vertical)
                        .lineLimit(dynamicTypeSize.isAccessibilitySize ? 1...2 : 2...5)
                        .accessibilityLabel(Text("messenger_composer"))
                        .accessibilityIdentifier(ConsoleAccessibilityID.messengerComposerField)
                }
                    .layoutPriority(1)
                Button {
                    Task { await viewModel.sendMessengerMessage() }
                } label: {
                    Label("messenger_send", systemImage: "paperplane.fill")
                        .labelStyle(.iconOnly)
                        .frame(minWidth: 44, minHeight: 44)
                        .contentShape(Rectangle())
                }
                .accessibilityLabel(Text("messenger_send"))
                .accessibilityIdentifier(ConsoleAccessibilityID.messengerSendButton)
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
        // Platform-neutral. This package also builds for macOS (Package.swift
        // declares .macOS(.v14)) where `Color(_:)` resolves to Color(NSColor)
        // and `.systemBackground` does not exist, so the bare UIKit literal
        // fails the `ios-app` job's `swift build`. Every semantic surface in
        // this file goes through an os-guarded helper for exactly this reason.
        .background(Color.opaqueConsoleNavigationBackground)
    }
}

struct MessengerThreadRow: View {
    let thread: MessengerThread
    let isSelected: Bool
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize

    var body: some View {
        if dynamicTypeSize.isAccessibilitySize {
            VStack(alignment: .leading, spacing: 6) {
                Text(messengerThreadDisplayTitle(thread))
                    .font(.headline)
                ConsoleChip(key: thread.kind.fieldLabelKey)
                memberCount
            }
        } else {
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline) {
                    Text(messengerThreadDisplayTitle(thread))
                        .font(.headline)
                    Spacer()
                    ConsoleChip(key: thread.kind.fieldLabelKey)
                }
                memberCount
            }
        }
    }

    private var memberCount: some View {
        Text(localizedString("messenger_member_count_format", thread.memberCount))
            .font(.footnote)
            .foregroundStyle(.primary)
    }
}

private func messengerThreadDisplayTitle(_ thread: MessengerThread) -> String {
    if let title = thread.trimmedTitle {
        return title
    }

    switch thread.kind {
    case .workOrder:
        if let identifier = thread.friendlyWorkOrderIdentifier {
            return localizedString("messenger_thread_work_order_format", identifier)
        }
        return localizedString("messenger_thread_work_order")
    case .team:
        return localizedString("messenger_thread_team")
    case .dm:
        return localizedString("messenger_thread_dm")
    case .group:
        return localizedString("messenger_thread_group")
    }
}

private extension MessengerThread {
    var trimmedTitle: String? {
        guard let title else { return nil }
        let trimmed = title.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    var friendlyWorkOrderIdentifier: String? {
        guard let workOrderID else { return nil }
        let trimmed = workOrderID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.isEmpty == false else { return nil }
        return UUID(uuidString: trimmed) == nil ? trimmed : nil
    }
}

struct MessengerMessageRow: View {
    let message: MessengerMessage
    let currentUserID: Components.Schemas.Uuid?
    let accessibilityIdentifier: String
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if currentUserID == message.senderID, message.readTargetCount > 0 {
                Text(localizedString("messenger_read_progress_format", message.readCount, message.readTargetCount))
                    .font(.caption)
                    .foregroundStyle(.primary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.opaqueConsoleCapsuleBackground, in: Capsule())
            }
            messageContent
        }
    }

    @ViewBuilder
    private var messageContent: some View {
        if dynamicTypeSize.isAccessibilitySize {
            VStack(alignment: .leading, spacing: 6) {
                bodyText
                timestampAndAttachment
            }
        } else {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                bodyText
                Spacer(minLength: 8)
                timestampAndAttachment
            }
        }
    }

    private var bodyText: some View {
        Text(message.body)
            .font(.body)
            .fixedSize(horizontal: false, vertical: true)
            .accessibilityIdentifier(accessibilityIdentifier)
    }

    private var timestampAndAttachment: some View {
        HStack(spacing: 6) {
            Text(message.sentAt.formatted(date: .abbreviated, time: .shortened))
                .font(.caption)
                .accessibilityIdentifier(ConsoleAccessibilityID.messengerMessageTimestamp(message.id))
            if message.attachmentEvidenceIDs.isEmpty == false {
                ConsoleChip(key: "messenger_attachment")
            }
        }
        .foregroundStyle(.primary)
        .fixedSize(horizontal: false, vertical: true)
    }
}

struct WorkOrderRow: View {
    let workOrder: TechnicianWorkOrder

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(workOrder.requestNo)
                    .font(.headline)
                Spacer()
                ConsoleChip(key: workOrder.priority.fieldLabelKey)
            }
            Text(localizedString("site_format", workOrder.customerName, workOrder.siteName))
                .font(.subheadline)
            Text(localizedString("equipment_format", workOrder.managementNo, workOrder.modelName))
                .font(.footnote)
                .foregroundStyle(.primary)
            HStack {
                ConsoleChip(key: workOrder.status.fieldLabelKey)
                ConsoleChip(key: workOrder.syncState.fieldLabelKey)
            }
        }
        .padding(.vertical, 6)
    }
}

struct WorkOrderDetailView: View {
    @ObservedObject var viewModel: ConsoleViewModel
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize

    var body: some View {
        NavigationStack {
            if let workOrder = viewModel.selectedWorkOrder {
                Form {
                    LocationConsentSection(viewModel: viewModel)

                    Section {
                        metadataRow("request_no", value: workOrder.requestNo)
                        metadataRow(
                            "equipment",
                            value: localizedString("equipment_format", workOrder.managementNo, workOrder.modelName)
                        )
                        metadataRow("site", value: workOrder.siteName)
                        if let symptom = workOrder.symptom {
                            metadataRow(
                                "symptom",
                                value: symptom,
                                labelIdentifier: ConsoleAccessibilityID.detailSymptomLabel,
                                valueIdentifier: ConsoleAccessibilityID.detailSymptomValue
                            )
                        }
                        if let targetDueAt = workOrder.targetDueAt {
                            metadataRow(
                                "target_due",
                                value: targetDueAt.formatted(date: .abbreviated, time: .shortened)
                            )
                        }
                        HStack {
                            ConsoleChip(key: workOrder.priority.fieldLabelKey)
                            ConsoleChip(key: workOrder.status.fieldLabelKey)
                                .accessibilityIdentifier(ConsoleAccessibilityID.detailStatus)
                            ConsoleChip(key: workOrder.syncState.fieldLabelKey)
                        }
                    }

                    Section {
                        Button {
                            Task { await viewModel.startSelectedWork() }
                        } label: {
                            Label("detail_start_work", systemImage: "play.fill")
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.detailStartWorkButton)
                    }

                    Section {
                        Picker("result_type", selection: $viewModel.resultType) {
                            ForEach(reportableResults, id: \.self) { result in
                                Text(LocalizedStringKey(result.fieldLabelKey)).tag(result)
                            }
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.detailResultTypePicker)
                        TextField(String(localized: "report_diagnosis"), text: $viewModel.diagnosis, axis: .vertical)
                            .lineLimit(3...6)
                            .accessibilityIdentifier(ConsoleAccessibilityID.detailDiagnosisField)
                        TextField(String(localized: "report_action"), text: $viewModel.actionTaken, axis: .vertical)
                            .lineLimit(3...6)
                            .accessibilityIdentifier(ConsoleAccessibilityID.detailActionTakenField)
                        Button {
                            Task { await viewModel.submitReport() }
                        } label: {
                            Label("submit_report", systemImage: "paperplane.fill")
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.detailSubmitReportButton)

                        if let messageKey = viewModel.messageKey {
                            Text(LocalizedStringKey(messageKey))
                                .accessibilityIdentifier(ConsoleAccessibilityID.detailMessage)
                        }
                    }

                    Section {
                        Button {
                            viewModel.isCameraPresented = true
                        } label: {
                            Label("capture_evidence", systemImage: "camera.fill")
                        }
                        .accessibilityIdentifier(ConsoleAccessibilityID.detailCaptureEvidenceButton)
                    }

                }
                .scrollDismissesKeyboard(.immediately)
                // Keep the audited detail text on an opaque semantic surface in
                // every appearance and Dynamic Type size. The foreground is set
                // explicitly by metadata helpers rather than inferred from a
                // translucent material behind the Form.
                .scrollContentBackground(.hidden)
                .background(Color.opaqueConsoleDetailBackground)
                .tint(.primary)
                .accessibilityIdentifier(ConsoleAccessibilityID.detailView)
                .navigationTitle(Text("detail_title"))
                .inlineNavigationTitle()
                .toolbar {
                    Button {
                        viewModel.closeDetail()
                    } label: {
                        Label("back", systemImage: "xmark")
                    }
                    .accessibilityIdentifier(ConsoleAccessibilityID.detailBackButton)
                }
            }
        }
    }

    private var reportableResults: [Components.Schemas.WorkResultType] {
        [.completed, .temporaryAction, .incomplete, .revisitRequired]
    }

    @ViewBuilder
    private func metadataRow(
        _ labelKey: LocalizedStringKey,
        value: String,
        labelIdentifier: String? = nil,
        valueIdentifier: String? = nil
    ) -> some View {
        let usesVerticalLayout = dynamicTypeSize.isAccessibilitySize
        let layout = usesVerticalLayout
            ? AnyLayout(VStackLayout(alignment: .leading, spacing: 4))
            : AnyLayout(HStackLayout(alignment: .firstTextBaseline, spacing: 12))
        layout {
            metadataText(labelKey, identifier: labelIdentifier)
            if usesVerticalLayout == false {
                Spacer(minLength: 12)
            }
            metadataValueText(value, alignLeading: usesVerticalLayout, identifier: valueIdentifier)
        }
    }

    @ViewBuilder
    private func metadataText(_ key: LocalizedStringKey, identifier: String?) -> some View {
        let text = Text(key)
            .font(.body)
            .foregroundStyle(.primary)
            .fixedSize(horizontal: false, vertical: true)
        if let identifier {
            text.accessibilityIdentifier(identifier)
        } else {
            text
        }
    }

    @ViewBuilder
    private func metadataValueText(
        _ value: String,
        alignLeading: Bool,
        identifier: String?
    ) -> some View {
        let text = Text(value)
            .font(.body)
            .foregroundStyle(.primary)
            .multilineTextAlignment(alignLeading ? .leading : .trailing)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: alignLeading ? .infinity : nil, alignment: alignLeading ? .leading : .trailing)
        if let identifier {
            text.accessibilityIdentifier(identifier)
        } else {
            text
        }
    }
}

struct LocationConsentSection: View {
    @ObservedObject var viewModel: ConsoleViewModel
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize

    private var state: Components.Schemas.LocationConsentState {
        viewModel.locationConsent?.state ?? .noRecord
    }

    @ViewBuilder
    var body: some View {
        Section {
            consentControls
        } header: {
            Text("location_consent_title")
                .foregroundStyle(.primary)
                .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentTitle)
        }
        .headerProminence(.increased)
    }

    @ViewBuilder
    private var consentControls: some View {
            consentValueRow(
                labelKey: "location_consent_state",
                labelIdentifier: ConsoleAccessibilityID.locationConsentStateLabel,
                value: localizedString(locationConsentStateKey(state)),
                identifier: ConsoleAccessibilityID.locationConsentStateValue
            )

            consentValueRow(
                labelKey: "location_consent_collection",
                labelIdentifier: ConsoleAccessibilityID.locationConsentCollectionLabel,
                value: localizedString(viewModel.locationConsent?.mayCollect == true ? "yes" : "no"),
                identifier: ConsoleAccessibilityID.locationConsentCollectionValue
            )

            switch state {
            case .noRecord:
                Button {
                    Task { await viewModel.grantLocationConsent() }
                } label: {
                    consentButtonLabel("location_consent_grant")
                }
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentGrantButton)
            case .withdrawn:
                Button {
                    Task { await viewModel.grantLocationConsent() }
                } label: {
                    consentButtonLabel("location_consent_regain")
                }
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentGrantButton)
            case .granted:
                Button {
                    Task { await viewModel.suspendLocationConsent() }
                } label: {
                    consentButtonLabel("location_consent_suspend")
                }
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentSuspendButton)

                withdrawButton
            case .suspended:
                Button {
                    Task { await viewModel.resumeLocationConsent() }
                } label: {
                    consentButtonLabel("location_consent_resume")
                }
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentResumeButton)

                withdrawButton
            }
    }

    @ViewBuilder
    private func consentValueRow(
        labelKey: LocalizedStringKey,
        labelIdentifier: String,
        value: String,
        identifier: String
    ) -> some View {
        let usesVerticalLayout = dynamicTypeSize.isAccessibilitySize
        let layout = usesVerticalLayout
            ? AnyLayout(VStackLayout(alignment: .leading, spacing: 4))
            : AnyLayout(HStackLayout(alignment: .firstTextBaseline, spacing: 12))
        layout {
            consentText(labelKey)
                .accessibilityIdentifier(labelIdentifier)
            if usesVerticalLayout == false {
                Spacer(minLength: 12)
            }
            consentValueText(value, identifier: identifier, alignLeading: usesVerticalLayout)
        }
    }

    private func consentText(_ key: LocalizedStringKey) -> some View {
        Text(key)
            .font(.body)
            .foregroundStyle(.primary)
            .fixedSize(horizontal: false, vertical: true)
    }

    private func consentValueText(_ value: String, identifier: String, alignLeading: Bool) -> some View {
        Text(value)
            .font(.body)
            .foregroundStyle(.primary)
            .multilineTextAlignment(alignLeading ? .leading : .trailing)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: alignLeading ? .infinity : nil, alignment: alignLeading ? .leading : .trailing)
            .accessibilityIdentifier(identifier)
    }

    private func consentButtonLabel(_ key: LocalizedStringKey) -> some View {
        Text(key)
            .font(.body)
            .fixedSize(horizontal: false, vertical: true)
    }

    private var withdrawButton: some View {
        Button(role: .destructive) {
            Task { await viewModel.withdrawLocationConsent() }
        } label: {
            consentButtonLabel("location_consent_withdraw")
        }
        .disabled(viewModel.isLoading)
        .accessibilityIdentifier(ConsoleAccessibilityID.locationConsentWithdrawButton)
    }
}

struct ConsoleChip: View {
    let key: String

    var body: some View {
        Text(LocalizedStringKey(key))
            .font(.caption)
            .foregroundStyle(.primary)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(Color.opaqueConsoleCapsuleBackground, in: Capsule())
    }
}

private extension View {
    @ViewBuilder
    func messengerNavigationBarBackground() -> some View {
        #if os(iOS)
        self
            .toolbarBackground(Color.opaqueConsoleNavigationBackground, for: .navigationBar)
            .toolbarBackground(.visible, for: .navigationBar)
        #else
        self
        #endif
    }

    @ViewBuilder
    func inlineNavigationTitle() -> some View {
        #if os(iOS)
        navigationBarTitleDisplayMode(.inline)
        #else
        self
        #endif
    }

    @ViewBuilder
    func workOrderDetailPresentation<Item: Identifiable, Presented: View>(
        item: Binding<Item?>,
        @ViewBuilder content: @escaping (Item) -> Presented
    ) -> some View {
        #if os(iOS)
        fullScreenCover(item: item, content: content)
        #else
        sheet(item: item, content: content)
        #endif
    }

}

private extension Color {
    /// Opaque semantic surfaces preserve text contrast for iOS accessibility
    /// audits. The Swift package also compiles this view on macOS, where UIKit
    /// colors are unavailable, so retain an opaque platform-neutral fallback.
    static var opaqueConsoleCapsuleBackground: Color {
        #if os(iOS)
        Color(uiColor: .systemGray5)
        #else
        .gray
        #endif
    }

    static var opaqueConsoleDetailBackground: Color {
        #if os(iOS)
        Color(uiColor: .systemGroupedBackground)
        #else
        .gray
        #endif
    }

    static var opaqueConsoleNavigationBackground: Color {
        #if os(iOS)
        Color(uiColor: .systemBackground)
        #else
        .white
        #endif
    }
}

private func localizedString(_ key: String, _ arguments: CVarArg...) -> String {
    let format = NSLocalizedString(key, bundle: fieldLocalizationBundle, comment: "")
    return String(format: format, locale: Locale.current, arguments: arguments)
}

private var fieldLocalizationBundle: Bundle {
    #if SWIFT_PACKAGE
    .module
    #else
    .main
    #endif
}

private func locationConsentStateKey(_ state: Components.Schemas.LocationConsentState) -> String {
    switch state {
    case .noRecord:
        return "location_consent_state_no_record"
    case .granted:
        return "location_consent_state_granted"
    case .suspended:
        return "location_consent_state_suspended"
    case .withdrawn:
        return "location_consent_state_withdrawn"
    }
}
