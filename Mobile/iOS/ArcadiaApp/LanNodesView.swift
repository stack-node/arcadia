import SwiftUI

private struct DiscoveredPeerRow: Identifiable {
    let id: String
    let hostname: String
}

private struct KnownPeerRow: Identifiable {
    let id: String
    let hostname: String
    let ip: String
    let status: String
}

/// LAN Nodes — GUI for `lan.scan` / `lan.node` via core `executeCommand`.
struct LanNodesView: View {
    let theme: AppTheme

    @State private var discovered: [DiscoveredPeerRow] = []
    @State private var known: [KnownPeerRow] = []
    @State private var rangeText = ""
    @State private var feedback = ""

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                TextField("Optional --range (CIDR or IP)", text: $rangeText)
                    .textFieldStyle(.roundedBorder)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()

                HStack(spacing: 12) {
                    Button("Scan") { runScan() }
                        .buttonStyle(.borderedProminent)
                        .tint(theme.accentTextColor)

                    Button("Refresh known") { refreshKnown() }
                        .buttonStyle(.bordered)

                    Button("Save connected (all)") {
                        runLanNode(["save"])
                    }
                    .buttonStyle(.bordered)
                }

                Group {
                    sectionTitle("Discovered")
                    if discovered.isEmpty {
                        secondary("Run scan to discover Arcadia LAN peers.")
                    } else {
                        ForEach(discovered) { row in
                            discoveredCard(row)
                        }
                    }
                }

                Group {
                    sectionTitle("Known nodes")
                    if known.isEmpty {
                        secondary("No peers in node state yet.")
                    } else {
                        ForEach(known) { row in
                            knownCard(row)
                        }
                    }
                }

                Group {
                    sectionTitle("Last command")
                    Text(feedback.isEmpty ? "—" : feedback)
                        .font(.body)
                        .foregroundStyle(theme.secondaryTextColor)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(14)
                        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: 14, style: .continuous)
                                .stroke(theme.cardStrokeColor, lineWidth: 1)
                        }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .onAppear {
            refreshKnown()
        }
    }

    private func sectionTitle(_ text: String) -> some View {
        Text(text)
            .font(.headline)
            .foregroundStyle(theme.primaryTextColor)
    }

    private func secondary(_ text: String) -> some View {
        Text(text)
            .font(.subheadline)
            .foregroundStyle(theme.secondaryTextColor)
    }

    private func discoveredCard(_ row: DiscoveredPeerRow) -> some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(row.hostname)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(theme.primaryTextColor)
                Text(row.id)
                    .font(.caption)
                    .foregroundStyle(theme.secondaryTextColor)
            }
            Spacer()
            Button("Pair") {
                runLanNode(["pair", row.id])
            }
            .buttonStyle(.bordered)
        }
        .padding(14)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }

    private func knownCard(_ row: KnownPeerRow) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(row.hostname)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(theme.primaryTextColor)
                    Text("\(row.ip) · \(row.status)")
                        .font(.caption)
                        .foregroundStyle(theme.secondaryTextColor)
                }
                Spacer()
            }

            HStack(spacing: 8) {
                switch row.status {
                case "pending-inbound":
                    Button("Accept") { runLanNode(["accept", row.ip]) }.buttonStyle(.bordered)
                    Button("Reject") { runLanNode(["reject", row.ip]) }.buttonStyle(.bordered)
                case "pending-outbound":
                    Button("Connect") { runLanNode(["connect", row.ip]) }.buttonStyle(.bordered)
                    Button("Reject") { runLanNode(["reject", row.ip]) }.buttonStyle(.bordered)
                case "connected":
                    Button("Save") { runLanNode(["save", row.ip]) }.buttonStyle(.bordered)
                default:
                    Button("Pair again") { runLanNode(["pair", row.ip]) }.buttonStyle(.bordered)
                }
            }
        }
        .padding(14)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
    }

    private func scanArgsFromField() -> [String] {
        let trimmed = rangeText.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            return []
        }
        return ["--range", trimmed]
    }

    private func runScan() {
        let out = executeLanScan(args: scanArgsFromField())
        feedback = out
        discovered = Self.parseScanOutput(out)
    }

    private func refreshKnown() {
        let out = executeLanStatus()
        feedback = out
        known = Self.parseStatusOutput(out)
    }

    private func runLanNode(_ args: [String]) {
        let out = executeCommand(
            token: "lan.node",
            args: args,
            context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil)
        )
        feedback = out
        known = Self.parseStatusOutput(executeLanStatus())
        if ["pair", "connect", "accept", "reject"].contains(args.first) {
            discovered = Self.parseScanOutput(executeLanScan(args: scanArgsFromField()))
        }
    }

    private func executeLanStatus() -> String {
        executeCommand(
            token: "lan.node",
            args: ["status"],
            context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil)
        )
    }

    private func executeLanScan(args: [String]) -> String {
        executeCommand(
            token: "lan.scan",
            args: args,
            context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil)
        )
    }

    /// Lines like `- 192.168.1.5 (host)` from `lan.scan`.
    private static func parseScanOutput(_ text: String) -> [DiscoveredPeerRow] {
        var rows: [DiscoveredPeerRow] = []
        for line in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let t = line.trimmingCharacters(in: .whitespaces)
            guard t.hasPrefix("- ") else { continue }
            let rest = String(t.dropFirst(2))
            guard let open = rest.lastIndex(of: "("),
                  let close = rest.lastIndex(of: ")"),
                  open < close
            else { continue }
            let ip = String(rest[..<open]).trimmingCharacters(in: .whitespaces)
            let hostname = String(rest[rest.index(after: open)..<close]).trimmingCharacters(in: .whitespaces)
            if !ip.isEmpty {
                rows.append(DiscoveredPeerRow(id: ip, hostname: hostname.isEmpty ? ip : hostname))
            }
        }
        rows
    }

    /// Lines like `- hostname (192.168.1.2) [alias] -> connected` from `lan.node status`.
    private static func parseStatusOutput(_ text: String) -> [KnownPeerRow] {
        var rows: [KnownPeerRow] = []
        for line in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let t = line.trimmingCharacters(in: .whitespaces)
            guard t.hasPrefix("- ") else { continue }
            guard let arrowRange = t.range(of: " -> ") else { continue }
            let left = String(t[..<arrowRange.lowerBound].dropFirst(2))
            let status = String(t[arrowRange.upperBound...]).trimmingCharacters(in: .whitespaces)
            guard let open = left.lastIndex(of: "("),
                  let close = left.lastIndex(of: ")"),
                  open < close
            else { continue }
            let hostname = String(left[..<open]).trimmingCharacters(in: .whitespaces)
            let ip = String(left[left.index(after: open)..<close]).trimmingCharacters(in: .whitespaces)
            if !ip.isEmpty {
                rows.append(KnownPeerRow(id: ip, hostname: hostname.isEmpty ? ip : hostname, ip: ip, status: status))
            }
        }
        rows
    }
}
