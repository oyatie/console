import XCTest

/// Messaging flow, driven against the real session + real backend. Covers the
/// exact seeded thread, persisted messages, search, and composer — the field
/// mechanic's day-to-day communication surface. Empty/first-row fallbacks are
/// forbidden because this job owns a deterministic database.
///
/// CI-ONLY (see `ConsoleCriticalPathUITests` for the why).
final class MessengerUITests: ConsoleUITestCase {
    private let seededMessageBody = "iOS CI 초기 메시지"
    private let sentMessageBody = "iOS CI 메신저 전송 지속성"

    private enum MessengerError: LocalizedError {
        case prerequisite(String)

        var errorDescription: String? {
            switch self {
            case let .prerequisite(message): message
            }
        }
    }

    private func openMessengerTab() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        // The Messenger tab item carries the Korean label 메신저.
        XCTAssertTrue(
            tapTab(KO.messengerTitle, destination: app.collectionViews[AID.messengerTab]),
            "Messenger tab 메신저 should select the exact Messenger destination."
        )
    }

    /// SwiftUI materializes only the visible part of its List. Messenger uses one
    /// collection view for the thread selector and selected-thread content, so a
    /// direct query for a lower message or composer is not proof it was absent.
    /// Normalize to the deterministic thread at the top after a relaunch, then
    /// take short controlled drags until the exact requested element is hittable.
    private func scrollToMessengerElement(
        _ element: XCUIElement,
        topSentinel: XCUIElement,
        timeout: TimeInterval = 15,
        maxSwipes: Int = 16
    ) -> XCUIElement? {
        let deadline = Date().addingTimeInterval(timeout)
        let list = app.collectionViews[AID.messengerTab]
        let initialProbe = min(timeout, 2)
        if element.waitForExistence(timeout: initialProbe), element.isHittable {
            return element
        }

        let listProbe = max(min(deadline.timeIntervalSinceNow, initialProbe), 0)
        guard list.waitForExistence(timeout: listProbe) else { return nil }

        for _ in 0..<maxSwipes {
            if topSentinel.exists, topSentinel.isHittable { break }
            guard Date() < deadline else { return nil }
            list.swipeDown()
            if element.exists, element.isHittable { return element }
        }

        let origin = list.coordinate(withNormalizedOffset: .zero)
        let trailingGutterX = max(list.frame.width * 0.9, 8)
        let dragStart = origin.withOffset(CGVector(dx: trailingGutterX, dy: list.frame.height * 0.72))
        let dragEnd = origin.withOffset(CGVector(dx: trailingGutterX, dy: list.frame.height * 0.48))
        for _ in 0..<maxSwipes {
            if element.exists, element.isHittable { return element }
            guard Date() < deadline else { return nil }
            dragStart.press(forDuration: 0.1, thenDragTo: dragEnd)
            let probe = max(min(deadline.timeIntervalSinceNow, 0.5), 0)
            if element.waitForExistence(timeout: probe), element.isHittable {
                return element
            }
        }
        return nil
    }

    private func openSeededThread() async throws -> XCUIElement {
        try await openMessengerTab()
        guard app.descendants(matching: .any)[AID.messengerSearchField].waitForExistence(timeout: 10) else {
            throw MessengerError.prerequisite("The Messenger search surface should render.")
        }

        let threadID = try UITestFixture.requiredID(UITestFixture.messengerThreadID)
        let thread = app.buttons[AID.messengerThreadRow(threadID)]
        guard thread.waitForExistence(timeout: 15) else {
            throw MessengerError.prerequisite("The exact isolated messenger thread must render.")
        }
        XCTAssertFalse(app.staticTexts[AID.messengerEmptyThreads].exists, "A seeded messenger fixture must never pass as an empty state.")
        thread.tap()

        let initialMessageID = try UITestFixture.requiredID(UITestFixture.messengerInitialMessageID)
        let message = app.descendants(matching: .any)[AID.messengerMessageRow(initialMessageID)]
        guard message.waitForExistence(timeout: 15) else {
            throw MessengerError.prerequisite("Selecting the exact thread must load the exact seeded message row.")
        }
        guard scrollToMessengerElement(app.staticTexts[seededMessageBody], topSentinel: thread) != nil else {
            throw MessengerError.prerequisite("The seeded message body must be visible after backend readback.")
        }
        return thread
    }

    /// The composer and send action are a persistent sibling of the List inside
    /// a `VStack` — deliberately NOT a bottom safe-area inset, which
    /// `hasUnobscuredTabContentHost` forbids in ConsoleViews.swift because the
    /// UIKit `contentLayoutGuide` seam owns bottom insets. Being outside the
    /// lazy List they are always materialized, so assert them directly:
    /// scrolling for them would mask a regression that put them back inside the
    /// List, which is precisely the defect this repairs.
    private func persistentComposer() -> XCUIElement {
        app.descendants(matching: .any)[AID.messengerComposerField]
    }

    private func persistentSendButton() -> XCUIElement {
        app.buttons[AID.messengerSendButton]
    }

    /// Wait for existence AND hittability together. Sampling `isHittable` once
    /// immediately after `waitForExistence` is a race: the bar exists at t≈0
    /// because it is persistent, so the sample lands while the software keyboard
    /// is still animating (notably right after `typeText`) or while the
    /// selected-thread insertion is mid-transaction. This keeps the
    /// never-scroll-for-it invariant while giving layout time to settle.
    private func waitUntilHittable(_ element: XCUIElement, timeout: TimeInterval = 15) -> Bool {
        let hittable = XCTNSPredicateExpectation(
            predicate: NSPredicate(format: "exists == true AND isHittable == true"),
            object: element
        )
        return XCTWaiter().wait(for: [hittable], timeout: timeout) == .completed
    }

    func testExactSeededMessengerThreadAndMessageRender() async throws {
        _ = try await openSeededThread()
        let composer = persistentComposer()
        XCTAssertTrue(
            composer.waitForExistence(timeout: 15),
            "Selecting the seeded thread must expose the persistent composer without scrolling."
        )
        XCTAssertTrue(
            waitUntilHittable(composer),
            "The persistent composer must become hittable without scrolling."
        )
    }

    func testMessengerSendSurvivesBackendRefresh() async throws {
        let thread = try await openSeededThread()

        let composer = persistentComposer()
        guard composer.waitForExistence(timeout: 15), waitUntilHittable(composer) else {
            XCTFail("Messenger composer must be persistently available for the exact seeded thread.")
            return
        }
        composer.tap()
        composer.typeText(sentMessageBody)

        let send = persistentSendButton()
        guard send.waitForExistence(timeout: 15), waitUntilHittable(send) else {
            XCTFail("Messenger send button must be persistently available for the exact seeded thread.")
            return
        }
        send.tap()

        guard scrollToMessengerElement(app.staticTexts[sentMessageBody], topSentinel: thread) != nil else {
            XCTFail("A successful send must render the server-returned message.")
            return
        }
        XCTAssertFalse(
            app.staticTexts[KO.messengerSendPending].exists,
            "A queued offline fallback is not a successful persisted send."
        )

        // Kill the app to discard the reducer's locally merged message state,
        // then prove a brand-new view model reads the sent body from the backend.
        app.terminate()
        let relaunchedThread = try await openSeededThread()
        XCTAssertNotNil(
            scrollToMessengerElement(app.staticTexts[sentMessageBody], topSentinel: relaunchedThread),
            "The sent message must be returned after a full app relaunch, proving backend persistence."
        )
        XCTAssertFalse(app.staticTexts[KO.operationFailed].exists, "Backend readback must not settle in a failure state.")
    }

    func testMessengerSearchUnmatchedQueryShowsRealNoResults() async throws {
        try await openMessengerTab()

        let search = app.descendants(matching: .any)[AID.messengerSearchField]
        guard search.waitForExistence(timeout: 10) else {
            XCTFail("Messenger search field should be present.")
            return
        }
        search.tap()
        // A query unlikely to match seeds the real no-results path.
        search.typeText("zzzznoresultsq")
        app.buttons[AID.messengerSearchButton].tap()

        XCTAssertTrue(
            app.staticTexts[AID.messengerSearchNoResults].waitForExistence(timeout: 10)
                || app.staticTexts[KO.messengerSearchNoResults].exists,
            "An unmatched search should surface 검색 결과가 없습니다."
        )
    }
}
