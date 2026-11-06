import SwiftUI

// MARK: - Liquid Glass Header

public struct GlassHeaderView: View {
    let title: String
    let subtitle: String

    public init(title: String, subtitle: String) {
        self.title = title
        self.subtitle = subtitle
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            VStack(spacing: 12) {
                Text(title)
                    .font(.largeTitle.bold())
                    .foregroundStyle(.primary)
                Text(subtitle)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 24)
            .glassEffect(.regular.interactive(), in: .rect(cornerRadius: 20))
        } else {
            VStack(spacing: 12) {
                Text(title)
                    .font(.largeTitle.bold())
                    .foregroundStyle(.primary)
                Text(subtitle)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 24)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
        }
    }
}

// MARK: - Liquid Glass Card

public struct GlassCardView: View {
    let title: String
    let value: String
    let icon: String

    public init(title: String, value: String, icon: String) {
        self.title = title
        self.value = value
        self.icon = icon
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundStyle(.blue)
                Text(value)
                    .font(.title.bold())
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .glassEffect(.regular, in: .rect(cornerRadius: 16))
        } else {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundStyle(.blue)
                Text(value)
                    .font(.title.bold())
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 16))
        }
    }
}

// MARK: - Liquid Glass Button

public struct GlassButtonView: View {
    let title: String
    let icon: String

    public init(title: String, icon: String) {
        self.title = title
        self.icon = icon
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            HStack {
                Image(systemName: icon)
                Text(title)
            }
            .font(.headline)
            .foregroundStyle(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .glassEffect(.regular.interactive(), in: .capsule)
        } else {
            HStack {
                Image(systemName: icon)
                Text(title)
            }
            .font(.headline)
            .foregroundStyle(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .background(.ultraThinMaterial, in: Capsule())
        }
    }
}

// MARK: - Liquid Glass Status Capsule

public struct GlassStatusCapsuleView: View {
    let title: String
    let icon: String

    public init(title: String, icon: String) {
        self.title = title
        self.icon = icon
    }

    private var iconColor: Color {
        switch icon {
        case "wifi":
            return .accentColor
        case "wifi.slash":
            return .secondary
        case "wifi.exclamationmark":
            return .red
        default:
            return .primary
        }
    }

    @ViewBuilder
    public var body: some View {
        let isExpanded = !title.isEmpty
        HStack(spacing: 5) {
            if icon == "spinner" {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle())
                    .tint(.accentColor)
                    .scaleEffect(0.65)
                    .frame(width: 13, height: 13)
            } else if !icon.isEmpty {
                Image(systemName: icon)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(iconColor)
                    .frame(width: 14, height: 14)
            }
            if isExpanded {
                Text(title)
                    .font(.system(size: 12.5, weight: .medium))
                    .foregroundStyle(.primary)
                    .lineLimit(1)
                    .transition(
                        .asymmetric(
                            insertion: .opacity
                                .combined(with: .scale(scale: 0.75, anchor: .leading))
                                .combined(with: .offset(x: -3)),
                            removal: .opacity
                                .combined(with: .scale(scale: 0.75, anchor: .leading))
                        )
                    )
            }
        }
        .padding(.horizontal, isExpanded ? 11 : 8.5)
        .frame(height: 30)
        .background(.ultraThinMaterial, in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.primary.opacity(0.08), lineWidth: 0.5)
        )
        .contentShape(Capsule())
        .animation(.spring(response: 0.38, dampingFraction: 0.72), value: isExpanded)
        .animation(.easeInOut(duration: 0.2), value: icon)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

// MARK: - Liquid Glass Navigation Bar

public struct GlassNavBarView: View {
    let title: String

    public init(title: String) {
        self.title = title
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            Text(title)
                .font(.headline)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .glassEffect(.regular, in: .rect(cornerRadius: 12))
        } else {
            Text(title)
                .font(.headline)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
        }
    }
}

// MARK: - Native Quick Look & File Share

#if canImport(UIKit)
import UIKit

// MARK: - Native Liquid Glass Back Button Styler

final class LiquidSpringButtonHandler: NSObject {
    static let shared = LiquidSpringButtonHandler()
    private let haptic = UIImpactFeedbackGenerator(style: .light)

    func attach(to button: UIButton) {
        button.removeTarget(self, action: nil, for: .allEvents)
        button.addTarget(self, action: #selector(touchDown(_:event:)), for: [.touchDown])
        button.addTarget(self, action: #selector(touchDrag(_:event:)), for: [.touchDragInside, .touchDragOutside])
        button.addTarget(self, action: #selector(touchUp(_:)), for: [.touchUpInside, .touchUpOutside, .touchCancel])
    }

    @objc private func touchDown(_ sender: UIButton, event: UIEvent) {
        haptic.prepare()
        haptic.impactOccurred()
        updateTransform(for: sender, event: event, isDown: true)
    }

    @objc private func touchDrag(_ sender: UIButton, event: UIEvent) {
        updateTransform(for: sender, event: event, isDown: true)
    }

    @objc private func touchUp(_ sender: UIButton) {
        // Elastic spring bounce back with liquid overshoot
        UIView.animate(
            withDuration: 0.50,
            delay: 0,
            usingSpringWithDamping: 0.45,
            initialSpringVelocity: 1.2,
            options: [.beginFromCurrentState, .allowUserInteraction]
        ) {
            sender.transform = .identity
            sender.layer.shadowOpacity = sender.traitCollection.userInterfaceStyle == .dark ? 0.25 : 0.08
        }
    }

    private func updateTransform(for sender: UIButton, event: UIEvent, isDown: Bool) {
        guard let touch = event.touches(for: sender)?.first else {
            UIView.animate(
                withDuration: 0.22,
                delay: 0,
                usingSpringWithDamping: 0.55,
                initialSpringVelocity: 0.8,
                options: [.beginFromCurrentState, .allowUserInteraction]
            ) {
                sender.transform = CGAffineTransform(scaleX: 0.88, y: 0.88)
            }
            return
        }

        let location = touch.location(in: sender)
        let center = CGPoint(x: sender.bounds.midX, y: sender.bounds.midY)
        let dx = location.x - center.x
        let dy = location.y - center.y

        // Rubber-band liquid pull: dampening displacement (max 8 points in touch direction)
        let maxPull: CGFloat = 8.0
        let distance = sqrt(dx * dx + dy * dy)
        let pullFactor = min(1.0, distance / 40.0)
        let pullX = (distance > 0 ? (dx / distance) : 0) * maxPull * pullFactor
        let pullY = (distance > 0 ? (dy / distance) : 0) * maxPull * pullFactor

        // Compress down to 0.88 and elastically stretch in the pull direction
        let baseScale: CGFloat = 0.88

        UIView.animate(
            withDuration: 0.16,
            delay: 0,
            usingSpringWithDamping: 0.6,
            initialSpringVelocity: 0.8,
            options: [.beginFromCurrentState, .allowUserInteraction]
        ) {
            let scaleTransform = CGAffineTransform(scaleX: baseScale, y: baseScale)
            let translationTransform = CGAffineTransform(translationX: pullX, y: pullY)
            sender.transform = scaleTransform.concatenating(translationTransform)
            sender.layer.shadowOpacity = sender.traitCollection.userInterfaceStyle == .dark ? 0.40 : 0.18
        }
    }
}

@_cdecl("eschool_style_liquid_glass_button")
public func eschool_style_liquid_glass_button(buttonPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(buttonPtr).takeUnretainedValue()
    NSLog("[LiquidGlass] Styling button with canonical Day-Showcase bordered Symbol::Back: %@", raw)

    guard let btn = raw as? UIButton else { return }

    // Remove legacy subviews if any
    let tag = 99881
    btn.viewWithTag(tag)?.removeFromSuperview()

    btn.isUserInteractionEnabled = true
    btn.backgroundColor = .clear
    btn.layer.backgroundColor = UIColor.clear.cgColor

    let applyStyle: (UIButton) -> Void = { button in
        // Canonical Apple bordered button: .bordered().icon(Symbol::Back).icon_only()
        var config = UIButton.Configuration.bordered()

        let symbolConfig = UIImage.SymbolConfiguration(pointSize: 17.0, weight: .semibold)
        config.image = UIImage(systemName: "chevron.backward", withConfiguration: symbolConfig)
        config.imagePlacement = .all
        config.title = nil
        config.cornerStyle = .capsule
        config.contentInsets = NSDirectionalEdgeInsets(top: 0, leading: 0, bottom: 0, trailing: 0)

        button.configuration = config
        button.layer.shadowOpacity = 0.0
    }

    btn.configurationUpdateHandler = { button in
        applyStyle(button)
    }
    applyStyle(btn)

    DispatchQueue.main.async {
        applyStyle(btn)
        LiquidSpringButtonHandler.shared.attach(to: btn)
    }
}

// MARK: - Hardware-Synchronized Keyboard Button Gluer

final class KeyboardGluer: NSObject {
    static let shared = KeyboardGluer()
    private weak var targetView: UIView?
    private var currentLift: CGFloat = 0

    override init() {
        super.init()
        let center = NotificationCenter.default
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillChangeFrameNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillShowNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillHideNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(windowDidBecomeKey(_:)),
            name: UIWindow.didBecomeKeyNotification,
            object: nil
        )
    }

    @objc private func windowDidBecomeKey(_ notification: Notification) {
        if let window = notification.object as? UIWindow {
            KeyboardDismisser.shared.install(on: window)
        }
    }

    func register(view: UIView) {
        NSLog("[KeyboardGluer] Registered view: %@", view)
        self.targetView = view
        if let window = view.window {
            KeyboardDismisser.shared.install(on: window)
        }
        eschool_install_tap_to_dismiss()
        if self.currentLift > 0 {
            view.transform = CGAffineTransform(translationX: 0, y: -self.currentLift)
        }
    }

    @objc private func keyboardWillChange(_ notification: Notification) {
        guard let view = self.targetView else {
            NSLog("[KeyboardGluer] targetView is nil on notification")
            return
        }

        guard let userInfo = notification.userInfo,
              let endFrameValue = userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue
        else { return }

        let rawEndFrame = endFrameValue.cgRectValue
        let duration = (userInfo[UIResponder.keyboardAnimationDurationUserInfoKey] as? NSNumber)?.doubleValue ?? 0.25
        let curveValue = (userInfo[UIResponder.keyboardAnimationCurveUserInfoKey] as? NSNumber)?.intValue ?? 7

        guard let window = view.window ?? UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .flatMap({ $0.windows })
            .first(where: { $0.isKeyWindow })
        else {
            NSLog("[KeyboardGluer] window is nil")
            return
        }

        KeyboardDismisser.shared.install(on: window)

        let endFrame = window.convert(rawEndFrame, from: nil)
        let screenHeight = window.bounds.height
        let safeAreaBottom = window.safeAreaInsets.bottom

        // Standard iOS portrait keyboard height is never > 390 pt.
        // During AutoFill sheet or FaceID/modal presentation, endFrame.origin.y can be 0 or small,
        // which would cause keyboardHeight to erroneously become screenHeight (800+ pt).
        let maxAllowedKeyboardHeight: CGFloat = min(390.0, screenHeight * 0.48)
        let keyboardHeight: CGFloat
        if endFrame.origin.y < screenHeight {
            let rawHeight = screenHeight - endFrame.origin.y
            if rawHeight > maxAllowedKeyboardHeight {
                // Clamped to standard keyboard height:
                keyboardHeight = min(336.0, screenHeight * 0.42)
            } else {
                keyboardHeight = max(0, rawHeight)
            }
        } else {
            keyboardHeight = 0
        }

        let lift: CGFloat = keyboardHeight > 0 ? max(0, keyboardHeight - safeAreaBottom + 8.0) : 0
        self.currentLift = lift

        NSLog("[KeyboardGluer] kbHeight=%.1f safeAreaBottom=%.1f lift=%.1f dur=%.2f", keyboardHeight, safeAreaBottom, lift, duration)

        let options = UIView.AnimationOptions(rawValue: UInt(curveValue << 16))

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: [options, .beginFromCurrentState, .allowUserInteraction],
            animations: {
                view.transform = lift > 0 ? CGAffineTransform(translationX: 0, y: -lift) : .identity
            },
            completion: nil
        )
    }
}

@_cdecl("eschool_glue_button_to_keyboard")
public func eschool_glue_button_to_keyboard(viewPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(viewPtr).takeUnretainedValue()
    KeyboardGluer.shared.register(view: raw)
}

// MARK: - Global Tap To Dismiss Keyboard

final class DismissKeyboardTapGesture: UITapGestureRecognizer {}

final class KeyboardDismisser: NSObject, UIGestureRecognizerDelegate {
    static let shared = KeyboardDismisser()

    func install(on window: UIWindow?) {
        guard let window = window else { return }
        if window.gestureRecognizers?.contains(where: { $0 is DismissKeyboardTapGesture }) == true {
            return
        }
        let tap = DismissKeyboardTapGesture(target: self, action: #selector(handleTap(_:)))
        tap.cancelsTouchesInView = false
        tap.requiresExclusiveTouchType = false
        tap.delegate = self
        window.addGestureRecognizer(tap)
        NSLog("[KeyboardDismisser] Installed tap gesture on window: %@", window)
    }

    @objc func handleTap(_ gesture: UITapGestureRecognizer) {
        guard let window = gesture.view as? UIWindow ?? UIApplication.shared.windows.first(where: { $0.isKeyWindow }) else { return }
        window.endEditing(true)
    }

    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch) -> Bool {
        // Never intercept touches intended for interactive controls (buttons, text fields, pickers)
        var v: UIView? = touch.view
        while let current = v {
            if current is UIControl || current is UITextField || current is UITextView {
                return false
            }
            v = current.superview
        }
        return true
    }

    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer) -> Bool {
        return true
    }
}

@_cdecl("eschool_install_tap_to_dismiss")
public func eschool_install_tap_to_dismiss() {
    DispatchQueue.main.async {
        let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
        let windows = scenes.flatMap { $0.windows }
        let allWindows = windows.isEmpty ? UIApplication.shared.windows : windows

        for window in allWindows {
            KeyboardDismisser.shared.install(on: window)
        }
    }
}
#if canImport(QuickLook)
import QuickLook

public final class EschoolPreviewItem: NSObject, QLPreviewItem {
    public let previewItemURL: URL?
    public let previewItemTitle: String?

    public init(url: URL, title: String? = nil) {
        self.previewItemURL = url
        self.previewItemTitle = title ?? url.lastPathComponent
        super.init()
    }
}

public final class EschoolQuickLookDataSource: NSObject, QLPreviewControllerDataSource {
    let item: EschoolPreviewItem

    public init(item: EschoolPreviewItem) {
        self.item = item
        super.init()
    }

    public func numberOfPreviewItems(in controller: QLPreviewController) -> Int {
        return 1
    }

    public func previewController(_ controller: QLPreviewController, previewItemAt index: Int) -> QLPreviewItem {
        return item
    }
}

private var qlDataSourceKey: UInt8 = 0
#endif

@_cdecl("eschool_quicklook_file")
public func eschool_quicklook_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        guard let windowScene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first,
              let rootVC = windowScene.windows.first(where: { $0.isKeyWindow })?.rootViewController else {
            return
        }

        var topVC = rootVC
        while let presented = topVC.presentedViewController {
            topVC = presented
        }

        let url = URL(fileURLWithPath: filePath)
        #if canImport(QuickLook)
        let previewItem = EschoolPreviewItem(url: url)
        let dataSource = EschoolQuickLookDataSource(item: previewItem)

        let qlController = QLPreviewController()
        qlController.dataSource = dataSource
        objc_setAssociatedObject(qlController, &qlDataSourceKey, dataSource, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)

        topVC.present(qlController, animated: true)
        #else
        let activityVC = UIActivityViewController(activityItems: [url], applicationActivities: nil)
        if let popover = activityVC.popoverPresentationController {
            popover.sourceView = topVC.view
        }
        topVC.present(activityVC, animated: true)
        #endif
    }
}

@_cdecl("eschool_share_file")
public func eschool_share_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        guard let windowScene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first,
              let rootVC = windowScene.windows.first(where: { $0.isKeyWindow })?.rootViewController else {
            return
        }

        var topVC = rootVC
        while let presented = topVC.presentedViewController {
            topVC = presented
        }

        let url = URL(fileURLWithPath: filePath)
        let activityVC = UIActivityViewController(activityItems: [url], applicationActivities: nil)
        if let popover = activityVC.popoverPresentationController {
            popover.sourceView = topVC.view
        }
        topVC.present(activityVC, animated: true)
    }
}

@_cdecl("eschool_open_url")
public func eschool_open_url(urlStr: UnsafePointer<CChar>) {
    let s = String(cString: urlStr)
    if let url = URL(string: s) {
        DispatchQueue.main.async {
            UIApplication.shared.open(url)
        }
    }
}
#elseif os(macOS)
import AppKit

@_cdecl("eschool_quicklook_file")
public func eschool_quicklook_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        NSWorkspace.shared.open(URL(fileURLWithPath: filePath))
    }
}

@_cdecl("eschool_share_file")
public func eschool_share_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        NSWorkspace.shared.open(URL(fileURLWithPath: filePath))
    }
}

@_cdecl("eschool_open_url")
public func eschool_open_url(urlStr: UnsafePointer<CChar>) {
    let s = String(cString: urlStr)
    if let url = URL(string: s) {
        DispatchQueue.main.async {
            NSWorkspace.shared.open(url)
        }
    }
}
#endif

