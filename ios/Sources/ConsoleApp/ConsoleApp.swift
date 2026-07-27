import SwiftUI

@main
struct ConsoleApp: App {
    private let container = ConsoleAppContainer.live()

    var body: some Scene {
        WindowGroup {
            ConsoleRootView(viewModel: ConsoleViewModel(container: container))
        }
    }
}
