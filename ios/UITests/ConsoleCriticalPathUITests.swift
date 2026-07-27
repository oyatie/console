import XCTest

/// Field mechanic critical path against the real isolated backend and a session
/// restored by the production Keychain path. CI-only.
final class ConsoleCriticalPathUITests: ConsoleUITestCase {
    func testAuthenticatedLaunchShowsTodayTabInKorean() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        XCTAssertTrue(app.tabBars.buttons[KO.todayTitle].waitForExistence(timeout: 10), "Today tab button 오늘 작업 should be visible after real-session restore.")
        XCTAssertFalse(app.textFields[AID.loginUserIDField].exists, "Login field must not be present once the real session is restored.")
    }

    func testDispatchListRendersDeterministicMechanicWorkOrder() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        XCTAssertTrue(app.collectionViews[AID.todayList].waitForExistence(timeout: 15), "Dispatch list should render.")
        let fixtureID = try UITestFixture.requiredID(UITestFixture.detailWorkOrderID)
        XCTAssertNotNil(
            scrollToWorkOrderRow(in: app, id: fixtureID, timeout: 15),
            "Dispatch tab must render the deterministic mechanic fixture; an empty state is a failed seed or API result, not a valid CI outcome."
        )
    }

    func testFullFixtureRowsRemainReachableAboveTabBar() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        assertTodayListEndsAtOrAboveTabBar(in: app)

        let list = app.collectionViews[AID.todayList]
        let fixtureKeys = [
            UITestFixture.startWorkOrderID,
            UITestFixture.reportWorkOrderID,
            UITestFixture.reportSuccessWorkOrderID,
            UITestFixture.adminApproveWorkOrderID,
            UITestFixture.adminRejectWorkOrderID,
        ]
        for fixtureKey in fixtureKeys {
            let fixtureID = try UITestFixture.requiredID(fixtureKey)
            guard let row = scrollToWorkOrderRow(in: app, id: fixtureID, timeout: 30) else {
                XCTFail("Full functional fixture row \(fixtureID) must be reachable above tab-bar chrome.")
                continue
            }
            XCTAssertNotNil(
                workOrderRowActivationPoint(in: app, row: row, list: list),
                "Full functional fixture row \(fixtureID) must expose a safe activation point."
            )
        }
    }

    func testOpenWorkOrderDetailAndAdvance() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.startWorkOrderID)

        guard let startWork = scrollToDetailElement(app.buttons[AID.detailStartWorkButton]) else {
            XCTFail("작업 시작 button should be reachable in the lazy detail form.")
            return
        }
        startWork.tap()

        // A successful mutation must be observable in the rendered detail,
        // not inferred from still being authenticated or from another IN_PROGRESS
        // row behind the sheet. The isolated fixture is seeded as assigned; the
        // production API transitions this exact detail to 진행 중.
        let detailStatus = app.descendants(matching: .any)[AID.detailStatus]
        guard detailStatus.waitForExistence(timeout: 15) else {
            XCTFail("The selected detail must expose its status through the stable detail.status identifier.")
            return
        }
        XCTAssertTrue(
            detailStatus.exists,
            "The selected detail must expose its status through the stable detail.status identifier."
        )
        XCTAssertEqual(
            detailStatus.label,
            KO.inProgress,
            "Starting the deterministic work order must visibly transition it to 진행 중."
        )
        XCTAssertFalse(
            app.staticTexts[AID.detailMessage].exists || app.staticTexts[KO.operationFailed].exists,
            "Starting work must not settle in the visible failure state."
        )
        XCTAssertFalse(app.textFields[AID.loginUserIDField].exists, "Advancing a work order must not drop the real session back to login.")

        let back = app.buttons[AID.detailBackButton]
        XCTAssertTrue(back.waitForExistence(timeout: 5), "Detail must expose a back control.")
        back.tap()
        XCTAssertTrue(app.collectionViews[AID.todayList].waitForExistence(timeout: 10), "Closing detail should return to the dispatch list.")
    }

    func testReportFormValidationSurfacesRealRequiredCopy() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.reportWorkOrderID)
        guard let submit = scrollToDetailElement(app.buttons[AID.detailSubmitReportButton]) else {
            XCTFail("보고 제출 button should be reachable in the lazy detail form.")
            return
        }
        submit.tap()
        XCTAssertTrue(
            app.staticTexts[KO.requiredField].waitForExistence(timeout: 5),
            "Submitting an empty report must render the exact required-field validation copy."
        )
        XCTAssertFalse(
            app.staticTexts[KO.operationFailed].exists,
            "Required-field validation must not collapse into the generic operation failure."
        )
    }

    func testReportSubmissionPersistsVisibleTerminalOutcome() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.reportSuccessWorkOrderID)

        guard let diagnosis = scrollToDetailElement(app.textFields[AID.detailDiagnosisField]) else {
            XCTFail("Report diagnosis field should be reachable in detail.")
            return
        }
        diagnosis.tap()
        diagnosis.typeText("iOS UI 보고 성공 진단\n")

        guard let actionTaken = scrollToDetailElement(app.textFields[AID.detailActionTakenField]) else {
            XCTFail("Report action field should be reachable in detail.")
            return
        }
        actionTaken.tap()
        actionTaken.typeText("iOS UI 보고 성공 조치\n")

        guard let submit = scrollToDetailElement(app.buttons[AID.detailSubmitReportButton]) else {
            XCTFail("보고 제출 button should remain reachable after completing the report fields.")
            return
        }
        submit.tap()

        let detail = app.descendants(matching: .any)[AID.detailView]
        let detailMessage = detail.descendants(matching: .any)[AID.detailMessage]
        guard let resolvedMessage = scrollToElement(
            detailMessage,
            in: detail,
            // The response is inserted immediately after the report controls.
            // Preserve that local scroll position instead of resetting the
            // full Form to its first section before searching forward.
            topSentinel: detail.buttons[AID.detailSubmitReportButton],
            timeout: 15
        ) else {
            XCTFail("Submitting the isolated report fixture must expose detail-owned live API feedback.")
            return
        }
        XCTAssertEqual(
            resolvedMessage.label,
            KO.reportSuccessMessage,
            "Submitting the isolated report fixture must prove the rendered live API success response."
        )

        let detailStatus = detail.descendants(matching: .any)[AID.detailStatus]
        guard let resolvedStatus = scrollToElement(
            detailStatus,
            in: detail,
            topSentinel: detail.staticTexts[KO.locationConsentTitle],
            timeout: 15
        ) else {
            XCTFail("Submitting the isolated report fixture must expose the detail-owned terminal status.")
            return
        }
        XCTAssertEqual(
            resolvedStatus.label,
            KO.reportSubmitted,
            "Submitting the isolated report fixture must visibly persist the 보고 완료 terminal outcome."
        )
        XCTAssertFalse(
            detail.staticTexts[KO.operationFailed].exists,
            "A successfully persisted report must not settle in the generic operation failure state."
        )
    }

    func testLocationConsentTransitionsPersistThroughTheRealBackend() async throws {
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.detailWorkOrderID)
        let detail = app.collectionViews[AID.detailView]
        XCTAssertTrue(detail.staticTexts[KO.locationConsentTitle].waitForExistence(timeout: 15), "GPS 위치 동의 section should render for the authenticated mechanic.")

        XCTAssertTrue(detailButton(AID.locationConsentGrantButton).waitForExistence(timeout: 5), "Location-consent controls must render.")
        XCTAssertTrue(waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentNoRecord), "The isolated backend must begin without a consent record.")
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.no), "No-record consent must prohibit GPS collection.")
        XCTAssertTrue(detailButton(AID.locationConsentGrantButton).isEnabled)
        XCTAssertFalse(detailButton(AID.locationConsentSuspendButton).exists)
        XCTAssertFalse(detailButton(AID.locationConsentResumeButton).exists)
        XCTAssertFalse(detailButton(AID.locationConsentWithdrawButton).exists)

        detailButton(AID.locationConsentGrantButton).tap()
        XCTAssertTrue(waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentGranted), "Grant must persist the consented state through the real API.")
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.yes), "Granted consent must permit GPS collection.")
        XCTAssertFalse(detailButton(AID.locationConsentGrantButton).exists)
        XCTAssertTrue(detailButton(AID.locationConsentSuspendButton).isEnabled)
        XCTAssertFalse(detailButton(AID.locationConsentResumeButton).exists)
        XCTAssertTrue(detailButton(AID.locationConsentWithdrawButton).isEnabled)

        // Discard all in-memory reducer state and prove GET status readback from
        // a newly launched app before continuing the state machine.
        app.terminate()
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.detailWorkOrderID)
        XCTAssertTrue(
            waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentGranted),
            "A fresh app launch must read the granted state back from the backend."
        )
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.yes), "Persisted granted consent must permit GPS collection after relaunch.")

        detailButton(AID.locationConsentSuspendButton).tap()
        XCTAssertTrue(waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentSuspended), "Suspend must persist the GPS-off state through the real API.")
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.no), "Suspended consent must prohibit GPS collection.")
        XCTAssertFalse(detailButton(AID.locationConsentGrantButton).exists)
        XCTAssertFalse(detailButton(AID.locationConsentSuspendButton).exists)
        XCTAssertTrue(detailButton(AID.locationConsentResumeButton).isEnabled)
        XCTAssertTrue(detailButton(AID.locationConsentWithdrawButton).isEnabled)

        detailButton(AID.locationConsentResumeButton).tap()
        XCTAssertTrue(waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentGranted), "Resume must restore the consented state through the real API.")
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.yes), "Resumed consent must permit GPS collection.")

        detailButton(AID.locationConsentWithdrawButton).tap()
        XCTAssertTrue(waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentWithdrawn), "Withdraw must persist the terminal revoked state through the real API.")
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.no), "Withdrawn consent must prohibit GPS collection.")
        XCTAssertTrue(detailButton(AID.locationConsentGrantButton).isEnabled)
        XCTAssertFalse(detailButton(AID.locationConsentSuspendButton).exists)
        XCTAssertFalse(detailButton(AID.locationConsentResumeButton).exists)
        XCTAssertFalse(detailButton(AID.locationConsentWithdrawButton).exists)
        XCTAssertFalse(app.staticTexts[KO.operationFailed].exists, "Every consent transition must complete without a visible failure state.")

        app.terminate()
        _ = try await launchApp()
        waitForAuthenticatedShell()
        try openSeededWorkOrder(fixtureKey: UITestFixture.detailWorkOrderID)
        XCTAssertTrue(
            waitForLabel(AID.locationConsentStateValue, containing: KO.locationConsentWithdrawn),
            "A fresh app launch must read the withdrawn terminal state back from the backend."
        )
        XCTAssertTrue(waitForLabel(AID.locationConsentCollectionValue, containing: KO.no), "Persisted withdrawn consent must prohibit GPS collection after relaunch.")
        XCTAssertFalse(app.staticTexts[KO.operationFailed].exists)
    }
}
