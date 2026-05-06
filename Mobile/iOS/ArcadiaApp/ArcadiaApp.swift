import SwiftUI

@main
struct ArcadiaApp: App {
    init() {
        let fm = FileManager.default
        if let appSupport = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first {
            let configRoot = appSupport
                .appendingPathComponent("Arcadia", isDirectory: true)
                .appendingPathComponent("Configuration", isDirectory: true)
            try? fm.createDirectory(at: configRoot, withIntermediateDirectories: true)
            setConfigRootPath(path: configRoot.path)
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
