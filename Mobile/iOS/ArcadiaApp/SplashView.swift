import SwiftUI

struct SplashView: View {
    let onComplete: () -> Void

    @State private var hillsProgress: Double = 0
    @State private var sunProgress: Double = 0
    @State private var archProgress: Double = 0
    @State private var starsProgress: Double = 0
    @State private var masterOpacity: Double = 0

    var body: some View {
        ZStack {
            Canvas { ctx, sz in
                drawBackground(ctx: &ctx, size: sz)
                drawHorizonGlow(ctx: &ctx, size: sz, t: sunProgress)
                drawArch(ctx: &ctx, size: sz, t: archProgress)
                drawStars(ctx: &ctx, size: sz, t: starsProgress)
                drawSun(ctx: &ctx, size: sz, t: sunProgress)
                drawHills(ctx: &ctx, size: sz, t: hillsProgress)
            }
            .ignoresSafeArea()
        }
        .opacity(masterOpacity)
        .onAppear(perform: startAnimation)
    }

    private func startAnimation() {
        Task {
            do {
                withAnimation(.easeIn(duration: 0.4)) { masterOpacity = 1 }

                try await Task.sleep(nanoseconds: 200_000_000)
                withAnimation(.interpolatingSpring(stiffness: 75, damping: 14)) {
                    hillsProgress = 1
                }

                try await Task.sleep(nanoseconds: 500_000_000)
                withAnimation(.easeOut(duration: 1.0)) { sunProgress = 1 }

                try await Task.sleep(nanoseconds: 600_000_000)
                withAnimation(.spring(response: 0.95, dampingFraction: 0.68)) {
                    archProgress = 1
                }

                try await Task.sleep(nanoseconds: 700_000_000)
                withAnimation(.easeIn(duration: 0.7)) { starsProgress = 1 }

                try await Task.sleep(nanoseconds: 600_000_000)
                withAnimation(.easeOut(duration: 0.55)) {
                    masterOpacity = 0
                }

                try await Task.sleep(nanoseconds: 650_000_000)
                onComplete()
            } catch {
                onComplete()
            }
        }
    }
}

// MARK: – Drawing helpers

private func drawBackground(ctx: inout GraphicsContext, size: CGSize) {
    let w = size.width, h = size.height
    let rect = Path(CGRect(origin: .zero, size: size))
    ctx.fill(rect, with: .linearGradient(
        Gradient(colors: [
            AppTheme.splashBackgroundTop,
            AppTheme.splashBackgroundMid,
            AppTheme.splashBackgroundHorizon,
            AppTheme.splashBackgroundBottom
        ]),
        startPoint: CGPoint(x: size.width / 2, y: 0),
        endPoint: CGPoint(x: size.width / 2, y: size.height)
    ))

    ctx.drawLayer { gc in
        gc.addFilter(.blur(radius: h * 0.075))
        gc.fill(
            Path(ellipseIn: CGRect(x: -w * 0.05, y: h * 0.20, width: w * 1.10, height: h * 0.48)),
            with: .color(AppTheme.splashBackgroundMid.opacity(0.34))
        )
        gc.fill(
            Path(ellipseIn: CGRect(x: -w * 0.08, y: h * 0.50, width: w * 1.16, height: h * 0.34)),
            with: .color(AppTheme.splashBackgroundHorizon.opacity(0.30))
        )
    }
}

private func drawHorizonGlow(ctx: inout GraphicsContext, size: CGSize, t: Double) {
    let w = size.width, h = size.height
    let cx = w * 0.5
    let cy = h * 0.690
    let glows: [(Double, Double, Double, Color)] = [
        (1.18, 0.46, 0.16, AppTheme.splashHorizonPink),
        (0.76, 0.30, 0.20, AppTheme.splashHorizonGold),
        (0.46, 0.20, 0.13, AppTheme.splashArchGlow),
        (0.30, 0.12, 0.10, AppTheme.splashStar)
    ]

    ctx.drawLayer { gc in
        gc.addFilter(.blur(radius: h * 0.045))
        for (widthScale, heightScale, alpha, color) in glows {
            let gw = w * widthScale
            let gh = h * heightScale
            let rect = CGRect(x: cx - gw * 0.5, y: cy - gh * 0.5, width: gw, height: gh)
            gc.fill(Path(ellipseIn: rect), with: .color(color.opacity(alpha * t)))
        }
    }
}

private func drawHills(ctx: inout GraphicsContext, size: CGSize, t: Double) {
    let w = size.width, h = size.height
    let offset = (1.0 - t) * h * 0.35

    func p(_ fx: Double, _ fy: Double) -> CGPoint {
        CGPoint(x: fx * w, y: fy * h + offset)
    }

    // Back hill
    var back = Path()
    back.move(to: p(0, 0.95))
    back.addCurve(to: p(0.50, 0.60), control1: p(0.15, 0.88), control2: p(0.35, 0.61))
    back.addCurve(to: p(1.0, 0.95), control1: p(0.65, 0.61), control2: p(0.85, 0.88))
    back.addLine(to: p(1.0, 1.10))
    back.addLine(to: p(0.0, 1.10))
    back.closeSubpath()
    ctx.fill(back, with: .color(AppTheme.splashHillBack.opacity(0.62)))

    // Left hill
    var left = Path()
    left.move(to: p(0, 0.95))
    left.addCurve(to: p(0.32, 0.640), control1: p(0.06, 0.88), control2: p(0.18, 0.655))
    left.addCurve(to: p(0.52, 0.745), control1: p(0.44, 0.635), control2: p(0.49, 0.720))
    left.addLine(to: p(0.52, 1.10))
    left.addLine(to: p(0.0, 1.10))
    left.closeSubpath()
    ctx.fill(left, with: .color(AppTheme.splashHillLeft.opacity(0.78)))

    // Right hill (mirror)
    var right = Path()
    right.move(to: p(1.0, 0.95))
    right.addCurve(to: p(0.68, 0.640), control1: p(0.94, 0.88), control2: p(0.82, 0.655))
    right.addCurve(to: p(0.48, 0.745), control1: p(0.56, 0.635), control2: p(0.51, 0.720))
    right.addLine(to: p(0.48, 1.10))
    right.addLine(to: p(1.0, 1.10))
    right.closeSubpath()
    ctx.fill(right, with: .color(AppTheme.splashHillRight.opacity(0.76)))

    var front = Path()
    front.move(to: p(0.0, 1.02))
    front.addCurve(to: p(0.50, 0.790), control1: p(0.18, 0.985), control2: p(0.34, 0.900))
    front.addCurve(to: p(1.0, 1.02), control1: p(0.66, 0.900), control2: p(0.82, 0.985))
    front.addLine(to: p(1.0, 1.10))
    front.addLine(to: p(0.0, 1.10))
    front.closeSubpath()
    ctx.fill(front, with: .color(AppTheme.splashHillFront.opacity(0.90)))
}

private func drawSun(ctx: inout GraphicsContext, size: CGSize, t: Double) {
    let w = size.width, h = size.height
    let cx = w * 0.5
    let finalY = h * 0.695
    let cy = h + (finalY - h) * t
    let r = max(splashSceneWidth(size) * 0.058, 26)

    ctx.drawLayer { gc in
        gc.addFilter(.blur(radius: r * 0.65))
        gc.fill(
            Path(ellipseIn: CGRect(x: cx - r * 7.0, y: cy - r * 7.0, width: r * 14.0, height: r * 14.0)),
            with: .radialGradient(
                Gradient(colors: [
                    AppTheme.splashHorizonGold.opacity(0.12 * t),
                    AppTheme.splashHorizonPink.opacity(0.04 * t),
                    AppTheme.splashHorizonPink.opacity(0)
                ]),
                center: CGPoint(x: cx, y: cy),
                startRadius: r * 0.6,
                endRadius: r * 7.0
            )
        )
    }

    for (rm, alpha, color) in AppTheme.splashSunLayers {
        let gr = r * rm
        let rect = CGRect(x: cx - gr, y: cy - gr, width: gr * 2, height: gr * 2)
        ctx.fill(
            Path(ellipseIn: rect),
            with: .color(color.opacity(alpha * t))
        )
    }
}

private func drawArch(ctx: inout GraphicsContext, size: CGSize, t: Double) {
    guard t > 0.001 else { return }
    let w = size.width, h = size.height
    let sceneW = splashSceneWidth(size)
    let apexX = w * 0.5
    let apexY = h * 0.195
    let baseY = h * 0.810
    let leftX = apexX - sceneW * 0.285
    let rightX = apexX + sceneW * 0.285

    func fp(_ x: Double, _ y: Double) -> CGPoint {
        return CGPoint(x: apexX + (x - apexX) * t, y: apexY + (y - apexY) * t)
    }

    func makeArch() -> Path {
        var p = Path()
        p.move(to: fp(leftX, baseY))
        p.addCurve(
            to: fp(apexX, apexY),
            control1: fp(leftX + sceneW * 0.035, h * 0.520),
            control2: fp(apexX - sceneW * 0.135, apexY)
        )
        p.addCurve(
            to: fp(rightX, baseY),
            control1: fp(apexX + sceneW * 0.135, apexY),
            control2: fp(rightX - sceneW * 0.035, h * 0.520)
        )
        return p
    }

    let arch = makeArch()
    let archWidth = min(max(sceneW * 0.078, 44), 98)

    ctx.drawLayer { gc in
        gc.addFilter(.blur(radius: archWidth * 0.45))
        gc.stroke(
            arch,
            with: .color(AppTheme.splashArchGlow.opacity(t * 0.22)),
            style: StrokeStyle(lineWidth: archWidth * 2.15, lineCap: .round, lineJoin: .round)
        )
        gc.stroke(
            arch,
            with: .color(AppTheme.splashHorizonPink.opacity(t * 0.16)),
            style: StrokeStyle(lineWidth: archWidth * 1.65, lineCap: .round, lineJoin: .round)
        )
        gc.stroke(
            arch,
            with: .color(AppTheme.splashArchGlow.opacity(t * 0.38)),
            style: StrokeStyle(lineWidth: archWidth * 1.35, lineCap: .round, lineJoin: .round)
        )
    }

    ctx.stroke(
        arch,
        with: .color(AppTheme.splashArchCore.opacity(t)),
        style: StrokeStyle(lineWidth: archWidth, lineCap: .round, lineJoin: .round)
    )
}

private func drawStars(ctx: inout GraphicsContext, size: CGSize, t: Double) {
    let w = size.width, h = size.height

    // (fx, fy, radius, delay, isSparkle)
    let stars: [(Double, Double, Double, Double, Bool)] = [
        (0.500, 0.380, 6.0, 0.00, true),
        (0.460, 0.295, 2.2, 0.15, false),
        (0.525, 0.275, 1.8, 0.25, false),
        (0.572, 0.345, 1.8, 0.35, false),
        (0.442, 0.418, 1.5, 0.10, false),
        (0.551, 0.448, 1.5, 0.20, false),
        (0.610, 0.318, 2.0, 0.30, false),
        (0.398, 0.362, 1.5, 0.40, false),
    ]

    for (fx, fy, r, delay, isSparkle) in stars {
        let denom = max(1.0 - delay, 0.1)
        let lt = min(max((t - delay) / denom, 0), 1)
        guard lt > 0 else { continue }
        let cx = fx * w
        let cy = fy * h

        if isSparkle {
            drawSparkle(ctx: &ctx, cx: cx, cy: cy, r: r, alpha: lt)
        } else {
            let rect = CGRect(x: cx - r, y: cy - r, width: r * 2, height: r * 2)
            ctx.fill(Path(ellipseIn: rect), with: .color(AppTheme.splashStar.opacity(lt)))
        }
    }
}

private func splashSceneWidth(_ size: CGSize) -> Double {
    min(size.width, size.height * 1.52)
}

private func drawSparkle(ctx: inout GraphicsContext, cx: Double, cy: Double, r: Double, alpha: Double) {
    // Glow behind sparkle
    ctx.drawLayer { gc in
        gc.addFilter(.blur(radius: r * 1.2))
        let gr = r * 2
        gc.fill(Path(ellipseIn: CGRect(x: cx - gr, y: cy - gr, width: gr * 2, height: gr * 2)),
                with: .color(AppTheme.splashStar.opacity(alpha * 0.5)))
    }

    for angleOffset in [0.0, Double.pi / 4] {
        var path = Path()
        let inner = r * 0.18
        let pts = 4
        for i in 0 ..< pts * 2 {
            let angle = angleOffset + Double(i) * .pi / Double(pts) - .pi / 2
            let rad = i % 2 == 0 ? r : inner
            let pt = CGPoint(x: cx + cos(angle) * rad, y: cy + sin(angle) * rad)
            if i == 0 { path.move(to: pt) } else { path.addLine(to: pt) }
        }
        path.closeSubpath()
        ctx.fill(path, with: .color(AppTheme.splashStar.opacity(alpha)))
    }
}
