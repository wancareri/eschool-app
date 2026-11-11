#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Generate high-resolution diagram images for the research paper.
Saved in resource/diagrams/.
"""

from PIL import Image, ImageDraw, ImageFont
import os

FONT_BOLD = "/usr/share/fonts/liberation/LiberationSans-Bold.ttf"
FONT_REG = "/usr/share/fonts/liberation/LiberationSans-Regular.ttf"
BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(BASE_DIR, "resource", "diagrams")
os.makedirs(OUT_DIR, exist_ok=True)

def f_bold(size):
    return ImageFont.truetype(FONT_BOLD, size)

def f_reg(size):
    return ImageFont.truetype(FONT_REG, size)

def draw_card(draw, x, y, w, h, bg_color, border_color, border_width=2, radius=12):
    draw.rounded_rectangle([x, y, x + w, y + h], radius=radius, fill=bg_color, outline=border_color, width=border_width)

def draw_arrow_down(draw, x, y1, y2, color="#3B82F6", width=3):
    draw.line([x, y1, x, y2], fill=color, width=width)
    draw.polygon([(x - 6, y2 - 8), (x + 6, y2 - 8), (x, y2 + 2)], fill=color)

def draw_arrow_right(draw, x1, y, x2, color="#3B82F6", width=3):
    draw.line([x1, y, x2, y], fill=color, width=width)
    draw.polygon([(x2 - 8, y - 6), (x2 - 8, y + 6), (x2 + 2, y)], fill=color)

# =============================================================================
# 1. ARCHITECTURE DIAGRAM
# =============================================================================
def make_arch_diagram():
    img = Image.new("RGB", (1300, 720), color="#FFFFFF")
    draw = ImageDraw.Draw(img)

    draw.text((650, 25), "Архитектурная схема программного комплекса «eschool-app»", font=f_bold(24), fill="#1E293B", anchor="mt")

    # Layer 1: UI Layer (Day Framework)
    draw_card(draw, 50, 70, 750, 160, "#F0F9FF", "#0284C7", 2)
    draw.text((70, 85), "1. Уровень представления (UI & Widgets Layer — Rust + Day Framework)", font=f_bold(18), fill="#0369A1")
    
    draw_card(draw, 70, 120, 210, 85, "#FFFFFF", "#BAE6FD", 1)
    draw.text((175, 130), "Pages (Экраны)", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((75, 155), "Login · Diary · Schedule\nQuarterStats · Settings", font=f_reg(13), fill="#475569", align="center")

    draw_card(draw, 310, 120, 230, 85, "#FFFFFF", "#BAE6FD", 1)
    draw.text((425, 130), "Custom Widgets", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((320, 155), "Dynamic Island Status\nLottieView · GradePeek", font=f_reg(13), fill="#475569", align="center")

    draw_card(draw, 570, 120, 210, 85, "#FFFFFF", "#BAE6FD", 1)
    draw.text((675, 130), "Native UIKit Bridge", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((580, 155), "Direct C-ABI / staticlib\n120 FPS Keyboard Anim", font=f_reg(13), fill="#475569", align="center")

    # Layer 2: State & Core Logic
    draw_card(draw, 50, 270, 750, 170, "#F5F3FF", "#7C3AED", 2)
    draw.text((70, 285), "2. Уровень реактивного состояния и логики (AppState Core)", font=f_bold(18), fill="#6D28D9")

    draw_card(draw, 70, 320, 210, 95, "#FFFFFF", "#DDD6FE", 1)
    draw.text((175, 330), "AppState (Signals)", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((75, 355), "Signal<is_authenticated>\nSignal<lessons>, <marks>\nFine-grained reactivity", font=f_reg(12), fill="#475569", align="center")

    draw_card(draw, 310, 320, 230, 95, "#FFFFFF", "#DDD6FE", 1)
    draw.text((425, 330), "Features Coordination", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((320, 355), "auth.rs (Refresh token)\ndiary.rs (Sync & parse)\ngrade_checker.rs (Watch)", font=f_reg(12), fill="#475569", align="center")

    draw_card(draw, 570, 320, 210, 95, "#FFFFFF", "#DDD6FE", 1)
    draw.text((675, 330), "Offline Cache Engine", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((580, 355), "Local storage snapshot\nGraceful offline mode\nZero-loss marks sync", font=f_reg(12), fill="#475569", align="center")

    # Layer 3: Network Crate (eschool-api)
    draw_card(draw, 50, 480, 750, 180, "#ECFDF5", "#059669", 2)
    draw.text((70, 495), "3. Сетевой уровень (Крейт crates/eschool-api на базе Reqwest)", font=f_bold(18), fill="#047857")

    draw_card(draw, 70, 530, 210, 105, "#FFFFFF", "#A7F3D0", 1)
    draw.text((175, 540), "OAuth 2.0 PKCE", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((75, 565), "SHA-256 code challenge\nToken exchange & refresh\nCookie jar session", font=f_reg(12), fill="#475569", align="center")

    draw_card(draw, 310, 530, 230, 105, "#FFFFFF", "#A7F3D0", 1)
    draw.text((425, 540), "REST Client & Endpoints", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((320, 565), "Typed URI builders\nBearer auth headers\nTimeout & retry logic", font=f_reg(12), fill="#475569", align="center")

    draw_card(draw, 570, 530, 210, 105, "#FFFFFF", "#A7F3D0", 1)
    draw.text((675, 540), "Serde JSON Models", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((580, 565), "DaySchedule, Lesson\nQuarterMarks, Bells\nStrict type validation", font=f_reg(12), fill="#475569", align="center")

    # Right column: System Security & Remote Services
    draw_card(draw, 840, 70, 410, 250, "#FEF3C7", "#D97706", 2)
    draw.text((1045, 90), "Системные сервисы безопасности (iOS)", font=f_bold(17), fill="#B45309", anchor="mt")
    draw_card(draw, 860, 130, 370, 75, "#FFFFFF", "#FDE68A", 1)
    draw.text((1045, 145), "Apple Keychain Services", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((1045, 170), "Аппаратное шифрование токенов (Secure Enclave)", font=f_reg(13), fill="#475569", anchor="mt")

    draw_card(draw, 860, 225, 370, 75, "#FFFFFF", "#FDE68A", 1)
    draw.text((1045, 240), "LocalAuthentication (Face ID / PIN)", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((1045, 265), "Биометрический вход и сторожевой автолок", font=f_reg(13), fill="#475569", anchor="mt")

    draw_card(draw, 840, 360, 410, 300, "#F1F5F9", "#475569", 2)
    draw.text((1045, 380), "Удаленные серверные платформы", font=f_bold(17), fill="#334155", anchor="mt")
    draw_card(draw, 860, 420, 370, 100, "#FFFFFF", "#CBD5E1", 1)
    draw.text((1045, 435), "Сервер авторизации РИОС", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((880, 460), "oauth.rios.unibel.by\nПроверка учетных данных, выдача токенов", font=f_reg(13), fill="#475569", align="center")

    draw_card(draw, 860, 540, 370, 100, "#FFFFFF", "#CBD5E1", 1)
    draw.text((1045, 555), "Серверы электронного дневника", font=f_bold(15), fill="#0F172A", anchor="mt")
    draw.text((880, 580), "diary.e-schools.by / api/v1\nРасписание, оценки, домашние задания", font=f_reg(13), fill="#475569", align="center")

    draw_arrow_down(draw, 425, 230, 265, "#0284C7")
    draw_arrow_down(draw, 425, 440, 475, "#7C3AED")
    draw_arrow_right(draw, 800, 150, 835, "#D97706")
    draw_arrow_right(draw, 800, 570, 835, "#059669")

    out_path = os.path.join(OUT_DIR, "fig1_architecture.png")
    img.save(out_path, dpi=(300, 300))
    print(f"Saved: {out_path}")

# =============================================================================
# 2. UI LOGIN SCREEN MOCKUP
# =============================================================================
def make_login_ui():
    img = Image.new("RGB", (1000, 680), color="#FFFFFF")
    draw = ImageDraw.Draw(img)

    draw.text((500, 20), "Визуальные состояния экрана авторизации и Lottie-персонажа", font=f_bold(22), fill="#1E293B", anchor="mt")

    # State 1: Intro
    draw_card(draw, 50, 70, 270, 560, "#F8FAFC", "#CBD5E1", 2, radius=24)
    draw_card(draw, 115, 85, 140, 18, "#0284C7", "#0284C7", 1, radius=9)
    draw.text((185, 130), "🦆", font=f_bold(50), fill="#F59E0B", anchor="mt")
    draw.text((185, 200), "Добро пожаловать!", font=f_bold(18), fill="#0F172A", anchor="mt")
    draw.text((185, 230), "Вход в электронный дневник", font=f_reg(13), fill="#64748B", anchor="mt")
    draw_card(draw, 75, 300, 220, 45, "#FFFFFF", "#E2E8F0", 1, radius=10)
    draw.text((90, 314), "Логин или телефон", font=f_reg(13), fill="#94A3B8")
    draw_card(draw, 75, 360, 220, 45, "#FFFFFF", "#E2E8F0", 1, radius=10)
    draw.text((90, 374), "Пароль", font=f_reg(13), fill="#94A3B8")
    draw_card(draw, 75, 440, 220, 48, "#0284C7", "#0284C7", 1, radius=12)
    draw.text((185, 454), "Войти", font=f_bold(15), fill="#FFFFFF", anchor="mt")
    draw.text((185, 595), "Состояние 1: Приветствие", font=f_bold(14), fill="#475569", anchor="mt")

    # State 2: Login focus
    draw_card(draw, 365, 70, 270, 560, "#F8FAFC", "#38BDF8", 2, radius=24)
    draw_card(draw, 430, 85, 140, 18, "#0284C7", "#0284C7", 1, radius=9)
    draw.text((500, 130), "👀", font=f_bold(50), fill="#F59E0B", anchor="mt")
    draw.text((500, 200), "Ввод логина", font=f_bold(18), fill="#0F172A", anchor="mt")
    draw.text((500, 230), "Утка следит за курсором", font=f_reg(13), fill="#0284C7", anchor="mt")
    draw_card(draw, 390, 300, 220, 45, "#FFFFFF", "#0284C7", 2, radius=10)
    draw.text((405, 314), "ivanov_11b|", font=f_bold(14), fill="#0F172A")
    draw_card(draw, 390, 360, 220, 45, "#FFFFFF", "#E2E8F0", 1, radius=10)
    draw.text((405, 374), "Пароль", font=f_reg(13), fill="#94A3B8")
    draw_card(draw, 390, 440, 220, 48, "#0284C7", "#0284C7", 1, radius=12)
    draw.text((500, 454), "Продолжить", font=f_bold(15), fill="#FFFFFF", anchor="mt")
    draw.text((500, 595), "Состояние 2: Фокус на логине", font=f_bold(14), fill="#475569", anchor="mt")

    # State 3: Password focus & Keyboard follower
    draw_card(draw, 680, 70, 270, 560, "#F8FAFC", "#818CF8", 2, radius=24)
    draw_card(draw, 745, 85, 140, 18, "#0284C7", "#0284C7", 1, radius=9)
    draw.text((815, 120), "🙈", font=f_bold(44), fill="#F59E0B", anchor="mt")
    draw.text((815, 180), "Ввод пароля", font=f_bold(17), fill="#0F172A", anchor="mt")
    draw.text((815, 205), "Утка закрыла глаза лапками", font=f_reg(12), fill="#6366F1", anchor="mt")
    draw_card(draw, 705, 245, 220, 42, "#FFFFFF", "#E2E8F0", 1, radius=10)
    draw.text((720, 258), "ivanov_11b", font=f_reg(13), fill="#64748B")
    draw_card(draw, 705, 295, 220, 42, "#FFFFFF", "#6366F1", 2, radius=10)
    draw.text((720, 308), "••••••••••••|", font=f_bold(14), fill="#0F172A")
    draw_card(draw, 705, 345, 220, 44, "#0284C7", "#0284C7", 1, radius=12)
    draw.text((815, 357), "Войти", font=f_bold(15), fill="#FFFFFF", anchor="mt")
    
    draw_card(draw, 690, 410, 250, 170, "#D1D5DB", "#9CA3AF", 1, radius=12)
    draw.text((710, 475), "[ Экранная клавиатура iOS ]\n120 FPS аппаратный подъем", font=f_bold(13), fill="#4B5563", align="center")
    draw.text((815, 595), "Состояние 3: Защита и клавиатура", font=f_bold(14), fill="#475569", anchor="mt")

    out_path = os.path.join(OUT_DIR, "fig2_login_flow.png")
    img.save(out_path, dpi=(300, 300))
    print(f"Saved: {out_path}")

# =============================================================================
# 3. UI DIARY SCREEN MOCKUP
# =============================================================================
def make_diary_ui():
    img = Image.new("RGB", (1000, 680), color="#FFFFFF")
    draw = ImageDraw.Draw(img)

    draw.text((500, 20), "Экранные формы электронного дневника и успеваемости", font=f_bold(22), fill="#1E293B", anchor="mt")

    # Screen 1: Diary lessons
    draw_card(draw, 100, 65, 360, 570, "#F8FAFC", "#0284C7", 2, radius=24)
    draw_card(draw, 180, 80, 200, 26, "#0F172A", "#0F172A", 1, radius=13)
    draw.text((280, 85), "Синхронизировано ✓", font=f_bold(12), fill="#10B981", anchor="mt")

    draw.text((120, 125), "Понедельник, 10 ноября", font=f_bold(18), fill="#0F172A")
    draw.text((120, 150), "11 «Б» класс · 9-я учебная неделя", font=f_reg(13), fill="#64748B")

    # Lesson 1
    draw_card(draw, 120, 180, 320, 95, "#FFFFFF", "#E2E8F0", 1, radius=12)
    draw.text((135, 195), "1. Алгебра и начала анализа", font=f_bold(15), fill="#0F172A")
    draw.text((135, 220), "Тема: Исследование функций", font=f_reg(13), fill="#64748B")
    draw.text((135, 245), "ДЗ: № 14.12, 14.15 (параграф 14)", font=f_reg(12), fill="#334155")
    draw_card(draw, 395, 195, 32, 32, "#DCFCE7", "#22C55E", 1, radius=8)
    draw.text((411, 203), "10", font=f_bold(14), fill="#15803D", anchor="mt")

    # Lesson 2
    draw_card(draw, 120, 285, 320, 95, "#FFFFFF", "#E2E8F0", 1, radius=12)
    draw.text((135, 300), "2. Физика", font=f_bold(15), fill="#0F172A")
    draw.text((135, 325), "Тема: Законы геометрической оптики", font=f_reg(13), fill="#64748B")
    draw.text((135, 350), "ДЗ: Решить задачи 1–3 на стр. 112", font=f_reg(12), fill="#334155")
    draw_card(draw, 395, 300, 32, 32, "#DBEAFE", "#3B82F6", 1, radius=8)
    draw.text((411, 308), "9", font=f_bold(14), fill="#1D4ED8", anchor="mt")

    # Lesson 3
    draw_card(draw, 120, 390, 320, 95, "#FFFFFF", "#E2E8F0", 1, radius=12)
    draw.text((135, 405), "3. Информатика", font=f_bold(15), fill="#0F172A")
    draw.text((135, 430), "Тема: Системное программирование на Rust", font=f_reg(13), fill="#64748B")
    draw.text((135, 455), "ДЗ: Подготовить проект к защите [файл]", font=f_reg(12), fill="#0284C7")
    draw_card(draw, 395, 405, 32, 32, "#DCFCE7", "#22C55E", 1, radius=8)
    draw.text((411, 413), "10", font=f_bold(14), fill="#15803D", anchor="mt")

    # Tab bar
    draw_card(draw, 100, 580, 360, 55, "#FFFFFF", "#E2E8F0", 1, radius=0)
    draw.text((115, 600), "Дневник", font=f_bold(12), fill="#0284C7")
    draw.text((195, 600), "Звонки", font=f_reg(12), fill="#64748B")
    draw.text((275, 600), "Учителя", font=f_reg(12), fill="#64748B")
    draw.text((355, 600), "Настройки", font=f_reg(12), fill="#64748B")

    draw.text((280, 645), "а) Экран дневника уроков и отметок", font=f_bold(14), fill="#1E293B", anchor="mt")

    # Screen 2: Quarter stats
    draw_card(draw, 540, 65, 360, 570, "#F8FAFC", "#7C3AED", 2, radius=24)
    draw_card(draw, 620, 80, 200, 26, "#0F172A", "#0F172A", 1, radius=13)
    draw.text((720, 85), "Офлайн: данные из кеша ⚠️", font=f_bold(11), fill="#F59E0B", anchor="mt")

    draw.text((560, 125), "Успеваемость за I четверть", font=f_bold(18), fill="#0F172A")
    draw.text((560, 150), "Средний балл четверти: 9.4", font=f_bold(14), fill="#7C3AED")

    subjects = [
        ("Математика", "9, 10, 9, 10, 10", "9.6", "10"),
        ("Русский язык", "8, 9, 9, 10", "9.0", "9"),
        ("Белорусский яз.", "9, 9, 10, 9", "9.25", "9"),
        ("Физика", "10, 9, 10, 9", "9.5", "10"),
        ("Информатика", "10, 10, 10, 10", "10.0", "10"),
        ("Химия", "9, 8, 9, 9", "8.75", "9"),
    ]

    y_pos = 185
    for s_name, marks, avg, final_m in subjects:
        draw_card(draw, 560, y_pos, 320, 52, "#FFFFFF", "#E2E8F0", 1, radius=10)
        draw.text((575, y_pos + 8), s_name, font=f_bold(14), fill="#0F172A")
        draw.text((575, y_pos + 28), f"Оценки: {marks}", font=f_reg(11), fill="#64748B")
        draw.text((790, y_pos + 16), f"ср. {avg}", font=f_reg(12), fill="#64748B")
        draw_card(draw, 840, y_pos + 10, 30, 30, "#EDE9FE", "#8B5CF6", 1, radius=6)
        draw.text((855, y_pos + 17), final_m, font=f_bold(13), fill="#6D28D9", anchor="mt")
        y_pos += 60

    draw_card(draw, 540, 580, 360, 55, "#FFFFFF", "#E2E8F0", 1, radius=0)
    draw.text((555, 600), "Дневник", font=f_reg(12), fill="#64748B")
    draw.text((635, 600), "Звонки", font=f_reg(12), fill="#64748B")
    draw.text((715, 600), "Учителя", font=f_reg(12), fill="#64748B")
    draw.text((795, 600), "Настройки", font=f_bold(12), fill="#7C3AED")

    draw.text((720, 645), "б) Сводка четвертных отметок и средний балл", font=f_bold(14), fill="#1E293B", anchor="mt")

    out_path = os.path.join(OUT_DIR, "fig3_diary_screens.png")
    img.save(out_path, dpi=(300, 300))
    print(f"Saved: {out_path}")

if __name__ == "__main__":
    make_arch_diagram()
    make_login_ui()
    make_diary_ui()
    print("All diagrams generated successfully.")
