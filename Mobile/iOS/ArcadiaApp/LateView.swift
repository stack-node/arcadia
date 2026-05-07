import SwiftUI

// ── Supporting types ──────────────────────────────────────────────────────────

private struct LateMessageRow: Identifiable, Decodable {
    let id: UInt64
    let username: String
    let body: String
    let timestamp: String
    let reactions: [LateReactionRow]
}

private struct LateReactionRow: Decodable {
    let emoji: String
    let count: Int
}

private struct LateNowPlayingRow: Decodable {
    var track: String = ""
    var artist: String = ""
    var album: String = ""
    var progressSec: Int = 0
    var durationSec: Int = 1
    var volumePct: Int = 0

    enum CodingKeys: String, CodingKey {
        case track, artist, album
        case progressSec = "progress_sec"
        case durationSec = "duration_sec"
        case volumePct  = "volume_pct"
    }
}

private struct LateVotesRow: Decodable {
    var lofi: Int = 0
    var ambient: Int = 0
    var classic: Int = 0
    var nextVoteAt: String = ""

    enum CodingKeys: String, CodingKey {
        case lofi, ambient, classic
        case nextVoteAt = "next_vote_at"
    }
}

private struct LateStatusPayload: Decodable {
    var connected: Bool = false
    var activeRoom: Int = 1
    var messages: Int = 0
    var onlineUsers: Int = 0
    var nowPlaying: LateNowPlayingRow = LateNowPlayingRow()
    var votes: LateVotesRow = LateVotesRow()
    var error: String?

    enum CodingKeys: String, CodingKey {
        case connected
        case activeRoom   = "active_room"
        case messages
        case onlineUsers  = "online_users"
        case nowPlaying   = "now_playing"
        case votes, error
    }
}

// ── View ──────────────────────────────────────────────────────────────────────

struct LateView: View {
    let theme: AppTheme

    @State private var messages: [LateMessageRow] = []
    @State private var composeText = ""
    @State private var nowPlaying = LateNowPlayingRow()
    @State private var votes = LateVotesRow()
    @State private var isConnected = false
    @State private var onlineUserCount = 0
    @State private var activeRoom = 1
    @State private var statusError: String?

    private let pollTimer = Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()

    var body: some View {
        VStack(spacing: 16) {
            playerBody
            chatBody
        }
        .padding(12)
        .onReceive(pollTimer) { _ in pollStatus() }
        .onAppear { pollStatus() }
    }

    // ── Chat ─────────────────────────────────────────────────────────────────

    private var chatBody: some View {
        VStack(spacing: 0) {
            connectionBanner
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 8) {
                        ForEach(messages) { msg in
                            messageRow(msg)
                        }
                        Color.clear.frame(height: 1).id("bottom")
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                }
                .onChange(of: messages.count) { _ in
                    withAnimation { proxy.scrollTo("bottom") }
                }
            }
            composeBar
        }
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14))
    }

    private var connectionBanner: some View {
        HStack {
            Circle()
                .fill(isConnected ? Color.green : Color.red)
                .frame(width: 8, height: 8)
            Text(isConnected ? "Connected · \(onlineUserCount) online" : "Disconnected")
                .font(.caption)
                .foregroundStyle(theme.secondaryTextColor)
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(.clear)
    }

    private func messageRow(_ msg: LateMessageRow) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text(msg.username)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(theme.accentTextColor)
                Text(formatTimestamp(msg.timestamp))
                    .font(.caption2)
                    .foregroundStyle(theme.secondaryTextColor)
            }
            Text(msg.body)
                .font(.callout)
                .foregroundStyle(theme.primaryTextColor)
            if !msg.reactions.isEmpty {
                HStack(spacing: 4) {
                    ForEach(msg.reactions, id: \.emoji) { r in
                        Text("\(r.emoji) \(r.count)")
                            .font(.caption2)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(theme.cardFillColor, in: Capsule())
                    }
                }
            }
        }
    }

    private var composeBar: some View {
        HStack(spacing: 8) {
            TextField("Type a message…", text: $composeText)
                .font(.callout)
                .foregroundStyle(theme.primaryTextColor)
                .padding(10)
                .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 10))
                .onSubmit { sendMessage() }
            Button(action: sendMessage) {
                Text("Send")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                    .background(Color.teal, in: RoundedRectangle(cornerRadius: 10))
            }
            .buttonStyle(.plain)
        }
        .padding(12)
        .background(theme.cardFillColor)
    }

    // ── Player ───────────────────────────────────────────────────────────────

    private var playerBody: some View {
        VStack(spacing: 12) {
            nowPlayingCard
            voteCard
        }
    }

    private var nowPlayingCard: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("NOW PLAYING")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.secondaryTextColor)
            Text(nowPlaying.track.isEmpty ? "—" : nowPlaying.track)
                .font(.title3.weight(.bold))
                .foregroundStyle(theme.primaryTextColor)
            if !nowPlaying.artist.isEmpty {
                Text("\(nowPlaying.artist) · \(nowPlaying.album)")
                    .font(.subheadline)
                    .foregroundStyle(theme.secondaryTextColor)
            }
            progressBar
            visualizerStrip
            HStack {
                Text(formatDuration(nowPlaying.progressSec))
                Spacer()
                Text("vol \(nowPlaying.volumePct)%  \(formatDuration(nowPlaying.durationSec))")
            }
            .font(.caption2)
            .foregroundStyle(theme.secondaryTextColor)
        }
        .padding(16)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14))
    }

    private var visualizerStrip: some View {
        HStack(alignment: .bottom, spacing: 5) {
            ForEach(0..<20, id: \.self) { idx in
                RoundedRectangle(cornerRadius: 2)
                    .fill(Color.teal.opacity(0.85))
                    .frame(
                        width: 6,
                        height: CGFloat(8 + ((nowPlaying.progressSec + idx * 7) % 20) * 2)
                    )
            }
        }
        .frame(maxWidth: .infinity, minHeight: 52, maxHeight: 52, alignment: .bottomLeading)
    }

    private var progressBar: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                Capsule().fill(theme.cardStrokeColor).frame(height: 4)
                let fraction = nowPlaying.durationSec > 0
                    ? CGFloat(nowPlaying.progressSec) / CGFloat(nowPlaying.durationSec)
                    : 0
                Capsule().fill(Color.teal).frame(width: geo.size.width * fraction, height: 4)
            }
        }
        .frame(height: 4)
    }

    private var voteCard: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("VOTE NEXT")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.secondaryTextColor)
            HStack(spacing: 10) {
                voteButton("Lofi", count: votes.lofi, genre: "lofi")
                voteButton("Ambient", count: votes.ambient, genre: "ambient")
                voteButton("Classic", count: votes.classic, genre: "classic")
            }
        }
        .padding(16)
        .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 14))
    }

    private func voteButton(_ label: String, count: Int, genre: String) -> some View {
        Button {
            runCommand("late.vote", args: [genre])
        } label: {
            VStack(spacing: 4) {
                Text(label).font(.caption.weight(.semibold))
                Text("\(count)").font(.caption2)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 8))
            .overlay {
                RoundedRectangle(cornerRadius: 8).stroke(theme.cardStrokeColor, lineWidth: 1)
            }
        }
        .buttonStyle(.plain)
        .foregroundStyle(theme.primaryTextColor)
    }

    // ── Actions ───────────────────────────────────────────────────────────────

    private func sendMessage() {
        let body = composeText.trimmingCharacters(in: .whitespaces)
        guard !body.isEmpty else { return }
        let _ = runCommand("late.send", args: ["\(activeRoom)", body])
        composeText = ""
    }

    private func pollStatus() {
        let raw = runCommand("late.status", args: [])
        guard let data = raw.data(using: .utf8),
              let payload = try? JSONDecoder().decode(LateStatusPayload.self, from: data) else {
            return
        }
        isConnected     = payload.connected
        activeRoom      = payload.activeRoom
        onlineUserCount = payload.onlineUsers
        nowPlaying      = payload.nowPlaying
        votes           = payload.votes
        statusError     = payload.error
    }

    @discardableResult
    private func runCommand(_ token: String, args: [String]) -> String {
        executeCommand(token: token, args: args, context: ExecutionContextFfi(netAs: nil, netTimeoutMs: nil))
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    private func formatTimestamp(_ ts: String) -> String {
        guard ts.count >= 16 else { return ts }
        return String(ts.dropFirst(11).prefix(5))
    }

    private func formatDuration(_ secs: Int) -> String {
        "\(secs / 60):\(String(format: "%02d", secs % 60))"
    }
}
