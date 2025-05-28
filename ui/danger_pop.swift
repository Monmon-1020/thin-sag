// ui/danger_pop.swift

import SwiftUI
import Network

@main
struct PopApp: App {
    @StateObject private var model = PopModel()

    var body: some Scene {
        WindowGroup {
            if let event = model.event {
                VStack(spacing: 20) {
                    Text("Dangerous action detected")
                        .font(.headline)
                    Text(event.path)
                        .font(.system(.caption))
                        .multilineTextAlignment(.center)
                        .padding(.horizontal)
                    HStack {
                        Button("Deny") {
                            model.reply(decision: "deny")
                        }
                        .keyboardShortcut(.cancelAction)
                        Button("Allow") {
                            model.reply(decision: "allow")
                        }
                        .keyboardShortcut(.defaultAction)
                    }
                }
                .padding()
                .frame(width: 400, height: 200)
            } else {
                Text("Waiting for request…")
                    .frame(width: 300, height: 100)
                    .onAppear { model.startListener() }
            }
        }
        .windowStyle(HiddenTitleBarWindowStyle())
    }
}

class PopModel: ObservableObject {
    struct Request: Codable {
        let rule_id: String
        let path: String
        let pid: Int
    }
    struct Response: Codable {
        let decision: String
        let cache_min: Int?
    }

    @Published var event: Request?
    private var listener: NWListener?
    private var connection: NWConnection?
    private let socketPath: String

    init() {
        let home = NSHomeDirectory()
        self.socketPath = "\(home)/.thin-sag/danger.sock"
    }

    /// ソケットリスナーを起動
    func startListener() {
        guard listener == nil else { return }

        // 既存ソケットファイルがあれば削除
        try? FileManager.default.removeItem(atPath: socketPath)

        // ① パラメータ作成
        let params = NWParameters()
        params.allowLocalEndpointReuse = true
        // ② UNIX ドメインソケット用エンドポイントを requiredLocalEndpoint に設定
        params.requiredLocalEndpoint = .unix(path: socketPath)

        // ③ listener 起動（using: のみ）
        do {
            listener = try NWListener(using: params)  // :contentReference[oaicite:0]{index=0}
        } catch {
            print("Listener init failed:", error)
            return
        }

        listener?.newConnectionHandler = { [weak self] conn in
            self?.accept(connection: conn)
        }
        listener?.start(queue: .main)
    }

    private func accept(connection conn: NWConnection) {
        self.connection = conn
        conn.start(queue: .global())
        receiveOneLine()
    }

    /// 一行（JSON）を受信
    private func receiveOneLine() {
        connection?.receive(minimumIncompleteLength: 1,
                            maximumLength: 4096) { [weak self] data, _, _, error in
            guard let data = data,
                  let line = String(data: data, encoding: .utf8)?
                                  .split(separator: "\n")
                                  .first
            else {
                print("Receive error:", error ?? "no data")
                return
            }

            if let req = try? JSONDecoder().decode(Request.self, from: Data(line.utf8)) {
                DispatchQueue.main.async {
                    self?.event = req
                }
            }
        }
    }

    /// ユーザー選択を JSON で返信しアプリ終了
    func reply(decision: String) {
        guard let conn = connection else { exit(1) }
        let resp = Response(decision: decision, cache_min: 30)
        guard let data = try? JSONEncoder().encode(resp) else { exit(1) }

        var msg = data
        msg.append(0x0A) // newline
        conn.send(content: msg, completion: .contentProcessed { _ in
            exit(0)
        })
    }
}
