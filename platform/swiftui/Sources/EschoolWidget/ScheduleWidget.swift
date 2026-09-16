import WidgetKit
import SwiftUI

// MARK: - Schedule Widget

struct ScheduleProvider: TimelineProvider {
    func placeholder(in context: Context) -> ScheduleEntry {
        ScheduleEntry(date: Date(), lessons: [
            LessonTime(subject: "Математика", time: "08:30", teacher: "Иванова И.И."),
            LessonTime(subject: "Русский язык", time: "09:25", teacher: "Петрова П.П."),
            LessonTime(subject: "Физика", time: "10:30", teacher: "Сидоров С.С."),
        ])
    }

    func getSnapshot(in context: Context, completion: @escaping (ScheduleEntry) -> Void) {
        completion(placeholder(in: context))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<ScheduleEntry>) -> Void) {
        // Load from shared UserDefaults (App Group)
        let defaults = UserDefaults(suiteName = "group.by.eschool.app.shared")
        let lessonsJSON = defaults?.string(forKey: "widget.schedule") ?? "[]"
        let lessons = (try? JSONDecoder().decode([LessonTime].self, from: lessonsJSON.data(using: .utf8)!)) ?? []

        let entry = ScheduleEntry(date: Date(), lessons: lessons)
        let nextUpdate = Calendar.current.date(byAdding: .hour, value: 1, to: Date())!
        let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
        completion(timeline)
    }
}

struct LessonTime: Codable, Identifiable {
    var id: String { "\(subject)-\(time)" }
    let subject: String
    let time: String
    let teacher: String
}

struct ScheduleEntry: TimelineEntry {
    let date: Date
    let lessons: [LessonTime]
}

struct ScheduleWidgetEntryView: View {
    var entry: ScheduleProvider.Entry
    @Environment(\.widgetFamily) var family

    var body: some View {
        switch family {
        case .systemSmall:
            smallView
        case .systemMedium:
            mediumView
        default:
            smallView
        }
    }

    var smallView: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "book.fill")
                    .foregroundStyle(.blue)
                Text("Расписание")
                    .font(.headline)
                Spacer()
                Text(todayString)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            if entry.lessons.isEmpty {
                VStack(spacing: 4) {
                    Image(systemName: "moon.zzz.fill")
                        .font(.title2)
                        .foregroundStyle(.secondary)
                    Text("Нет уроков")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ForEach(entry.lessons.prefix(4)) { lesson in
                    HStack(spacing: 8) {
                        Text(lesson.time)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .frame(width: 40, alignment: .leading)
                        Text(lesson.subject)
                            .font(.caption)
                            .lineLimit(1)
                    }
                }

                if entry.lessons.count > 4 {
                    Text("+\(entry.lessons.count - 4) ещё")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(12)
    }

    var mediumView: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "book.fill")
                    .foregroundStyle(.blue)
                Text("Расписание на сегодня")
                    .font(.headline)
                Spacer()
                Text(todayString)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            if entry.lessons.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "moon.zzz.fill")
                        .font(.title)
                        .foregroundStyle(.secondary)
                    Text("Нет уроков сегодня")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ForEach(entry.lessons.prefix(6)) { lesson in
                    HStack(spacing: 12) {
                        Text(lesson.time)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(width: 50, alignment: .leading)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(lesson.subject)
                                .font(.subheadline)
                                .lineLimit(1)
                            Text(lesson.teacher)
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                    }
                }

                if entry.lessons.count > 6 {
                    Text("+\(entry.lessons.count - 6) ещё")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(12)
    }

    var todayString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "dd.MM"
        return formatter.string(from: Date())
    }
}

@main
struct ScheduleWidget: Widget {
    let kind = "ScheduleWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: ScheduleProvider()) { entry in
            ScheduleWidgetEntryView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName("Расписание")
        .description("Расписание уроков на сегодня")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}
