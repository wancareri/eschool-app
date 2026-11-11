#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Генератор презентации (10 слайдов 16:9) для исследовательской работы.
Автор: Кусков Станислав Андреевич, 11 «Б» класс, ГУО «Гимназия № 21 г. Минска».
Выходные файлы:
  - /home/kayzer/Downloads/Презентация_Кусков_OpenSchool.pptx
  - /home/kayzer/Downloads/Презентация_Кусков_OpenSchool.pdf
"""

import os
import base64
import subprocess

OUTPUT_DIR = "/home/kayzer/Downloads"
OUTPUT_PPTX = os.path.join(OUTPUT_DIR, "Презентация_Кусков_OpenSchool.pptx")
OUTPUT_PDF = os.path.join(OUTPUT_DIR, "Презентация_Кусков_OpenSchool.pdf")
TEMP_FODP = "/tmp/presentation_openschool.fodp"

DIAGRAMS_DIR = "/home/kayzer/PetProjects/eschool-app/resource/diagrams"

def get_b64(path):
    with open(path, "rb") as f:
        return base64.b64encode(f.read()).decode("ascii")

def escape_xml(s):
    return (str(s).replace("&", "&amp;")
                  .replace("<", "&lt;")
                  .replace(">", "&gt;")
                  .replace('"', "&quot;")
                  .replace("'", "&apos;"))

def build_presentation():
    print("Loading image assets...")
    img_arch_b64 = get_b64(os.path.join(DIAGRAMS_DIR, "fig1_architecture.png"))
    img_login_b64 = get_b64(os.path.join(DIAGRAMS_DIR, "fig2_login_flow.png"))
    img_diary_b64 = get_b64(os.path.join(DIAGRAMS_DIR, "fig3_diary_screens.png"))

    slides = []

    # =========================================================================
    # СЛАЙД 1: ТИТУЛЬНЫЙ (Dark Navy Theme)
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide1" draw:master-page-name="MasterDark">
        <!-- Фон -->
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_dark"/>
        
        <!-- Верхний бейдж -->
        <draw:rect svg:x="2cm" svg:y="1.5cm" svg:width="15cm" svg:height="0.9cm" draw:style-name="badge_cyan">
          <draw:text-box>
            <text:p text:style-name="badge_text">МИНСКИЙ ГОРОДСКОЙ КОНКУРС ИССЛЕДОВАТЕЛЬСКИХ РАБОТ • СЕКЦИЯ «ИНФОРМАТИКА»</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Название проекта -->
        <draw:frame svg:x="2cm" svg:y="2.8cm" svg:width="24cm" svg:height="3.2cm">
          <draw:text-box>
            <text:p text:style-name="title_h1">Разработка кроссплатформенного приложения «eschool-app» для доступа к электронному дневнику на языке Rust с использованием UI-фреймворка Day</text:p>
            <text:p text:style-name="title_sub">Публичный бренд: «OpenSchool» • Электронный дневник нового поколения</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- Карточка автора и руководителя -->
        <draw:rect svg:x="2cm" svg:y="7.2cm" svg:width="14cm" svg:height="4.2cm" draw:style-name="card_dark">
          <draw:text-box>
            <text:p text:style-name="card_h_cyan">АВТОР ИССЛЕДОВАНИЯ:</text:p>
            <text:p text:style-name="card_text_white">Кусков Станислав Андреевич</text:p>
            <text:p text:style-name="card_text_muted">учащийся 11 «Б» класса ГУО «Гимназия № 21 г. Минска»</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="card_h_cyan">НАУЧНЫЙ РУКОВОДИТЕЛЬ:</text:p>
            <text:p text:style-name="card_text_white">Кузнецова Ольга Михайловна</text:p>
            <text:p text:style-name="card_text_muted">учитель информатики высшей категории</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Правая колонка с тегами стека -->
        <draw:rect svg:x="16.5cm" svg:y="7.2cm" svg:width="9.5cm" svg:height="4.2cm" draw:style-name="card_dark_border">
          <draw:text-box>
            <text:p text:style-name="card_h_emerald">КЛЮЧЕВЫЕ ТЕХНОЛОГИИ:</text:p>
            <text:p text:style-name="list_bullet_white">• Системный язык Rust 2024 (Zero-Cost)</text:p>
            <text:p text:style-name="list_bullet_white">• Декларативный нативный фреймворк Day</text:p>
            <text:p text:style-name="list_bullet_white">• Прямая компиляция в нативный Apple UIKit</text:p>
            <text:p text:style-name="list_bullet_white">• Аппаратный анклав Apple Secure Enclave</text:p>
            <text:p text:style-name="list_bullet_white">• Честная архитектура 100% Offline-First</text:p>
            <text:p text:style-name="list_bullet_white">• Открытый исходный код (Open Source)</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Нижняя плашка -->
        <draw:frame svg:x="2cm" svg:y="14.2cm" svg:width="24cm" svg:height="0.8cm">
          <draw:text-box>
            <text:p text:style-name="footer_dark">ГУО «Гимназия № 21 г. Минска» • Минск, 2026</text:p>
          </draw:text-box>
        </draw:frame>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 2: ПРОБЛЕМАТИКА ОФИЦИАЛЬНОГО СЕРВИСА
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide2" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">АКТУАЛЬНОСТЬ И ПРОБЛЕМАТИКА ОФИЦИАЛЬНОГО СЕРВИСА</text:p>
            <text:p text:style-name="slide_subtitle">Почему официальная веб-версия diary.e-schools.by вызывает массовые нарекания</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- 4 Карточки проблем -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="4.4cm" draw:style-name="card_red">
          <draw:text-box>
            <text:p text:style-name="card_h_red">1. Вечерние «Технические работы» (502/503)</text:p>
            <text:p text:style-name="card_p">Регулярные серверные регламентные работы приходятся на пиковые часы выполнения домашних заданий (18:00–22:00). Сервер выдает ошибку, блокируя доступ к расписанию и дневнику.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="4.4cm" draw:style-name="card_red">
          <draw:text-box>
            <text:p text:style-name="card_h_red">2. Отсутствие нативного мобильного приложения</text:p>
            <text:p text:style-name="card_p">В App Store и Google Play нет официального приложения. Школьники и родители вынуждены использовать тяжеловесный мобильный веб-сайт через браузер.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="1.5cm" svg:y="8.0cm" svg:width="12cm" svg:height="4.4cm" draw:style-name="card_amber">
          <draw:text-box>
            <text:p text:style-name="card_h_amber">3. Потеря отметок и сбои синхронизации</text:p>
            <text:p text:style-name="card_p">При слабом интернет-соединении в школьных зданиях сайт сбоит: свежие оценки не отображаются или пропадают при перезагрузке страницы, а сессии постоянно сбрасываются.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="8.0cm" svg:width="12cm" svg:height="4.4cm" draw:style-name="card_amber">
          <draw:text-box>
            <text:p text:style-name="card_h_amber">4. Отсутствие кеша и тяжелый трафик</text:p>
            <text:p text:style-name="card_p">Каждая страница сайта весит от 3 до 8 МБ, создавая задержки рендеринга до 5 секунд и перерасходуя лимиты мобильного трафика. Без сети экран остаётся белым.</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Нижний вывод -->
        <draw:rect svg:x="1.5cm" svg:y="13.0cm" svg:width="25cm" svg:height="1.6cm" draw:style-name="card_blue_accent">
          <draw:text-box>
            <text:p text:style-name="banner_text">РЕШЕНИЕ: Разработка независимого нативного клиента «OpenSchool» на системном языке Rust с честной архитектурой Offline-First и мгновенным откликом.</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 3: ЦЕЛЬ И ЗАДАЧИ
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide3" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">ЦЕЛЬ, ОБЪЕКТ И ЗАДАЧИ ИССЛЕДОВАНИЯ</text:p>
            <text:p text:style-name="slide_subtitle">Научно-практические ориентиры разработки программного комплекса</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- Блок Цели -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="25cm" svg:height="2.2cm" draw:style-name="card_goal">
          <draw:text-box>
            <text:p text:style-name="card_h_blue">ЦЕЛЬ РАБОТЫ:</text:p>
            <text:p text:style-name="card_goal_text">Исследование возможностей системного языка Rust и декларативного фреймворка Day для проектирования и практической реализации первого высокопроизводительного, защищенного и автономного кроссплатформенного клиента школьного дневника «OpenSchool».</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Сетка 6 Задач -->
        <draw:rect svg:x="1.5cm" svg:y="5.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">01</text:p>
            <text:p text:style-name="task_h">Сетевой стек eschool-api</text:p>
            <text:p text:style-name="task_p">Реализация модуля безопасной авторизации по протоколу OAuth 2.0 PKCE и отказоустойчивой десериализации.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="10.0cm" svg:y="5.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">02</text:p>
            <text:p text:style-name="task_h">Аппаратная безопасность</text:p>
            <text:p text:style-name="task_p">Интеграция системных аппаратных хранилищ: Apple Secure Enclave, Android Keystore, Windows TPM 2.0.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="18.5cm" svg:y="5.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">03</text:p>
            <text:p text:style-name="task_h">Реактивный нативный UI</text:p>
            <text:p text:style-name="task_p">Трансляция интерфейса в системный UIKit / Material без веб-оболочек с кадровой частотой 120 FPS.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="1.5cm" svg:y="9.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">04</text:p>
            <text:p text:style-name="task_h">Архитектура Offline-First</text:p>
            <text:p text:style-name="task_p">Разработка многоуровневого транзакционного локального кеша уроков, домашних заданий и четвертных баллов.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="10.0cm" svg:y="9.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">05</text:p>
            <text:p text:style-name="task_h">Биометрия и автоблокировка</text:p>
            <text:p text:style-name="task_p">Внедрение Face ID / Touch ID и посоленного PIN-кода со сторожевым таймером блокировки при сворачивании.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="18.5cm" svg:y="9.8cm" svg:width="8cm" svg:height="3.6cm" draw:style-name="card_task">
          <draw:text-box>
            <text:p text:style-name="task_num">06</text:p>
            <text:p text:style-name="task_h">Натурное тестирование</text:p>
            <text:p text:style-name="task_p">Практическая апробация на парке устройств учащихся 11 классов гимназии № 21 и подготовка к релизу.</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 4: ТЕХНОЛОГИЧЕСКИЙ СТЕК (Rust + Day vs Flutter / React Native)
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide4" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">ТЕХНОЛОГИЧЕСКИЙ СТЕК: ПРЕИМУЩЕСТВА RUST + DAY</text:p>
            <text:p text:style-name="slide_subtitle">Почему традиционные фреймворки уступают системному подходу</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- Колонка 1: Rust + Day (Победитель) -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="8.5cm" svg:height="10.8cm" draw:style-name="card_winner">
          <draw:text-box>
            <text:p text:style-name="stack_badge_green">НАШ ВЫБОР</text:p>
            <text:p text:style-name="stack_h1">Rust + Day</text:p>
            <text:p text:style-name="stack_lead">Прямая нативная компиляция</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="stack_check">✔ Память: всего 25–30 МБ</text:p>
            <text:p text:style-name="stack_check">✔ Плавность: 120 FPS ProMotion</text:p>
            <text:p text:style-name="stack_check">✔ Без сборщика мусора (Zero GC)</text:p>
            <text:p text:style-name="stack_check">✔ Безопасность памяти компилятором</text:p>
            <text:p text:style-name="stack_check">✔ Нативные системные UIKit виджеты</text:p>
            <text:p text:style-name="stack_check">✔ 100% переносимая бизнес-логика</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="stack_note">Идеальная отзывчивость даже на бюджетных процессорах.</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Колонка 2: Flutter -->
        <draw:rect svg:x="10.5cm" svg:y="3.2cm" svg:width="7.8cm" svg:height="10.8cm" draw:style-name="card_competitor">
          <draw:text-box>
            <text:p text:style-name="stack_badge_gray">ФРЕЙМВОРК</text:p>
            <text:p text:style-name="stack_h2">Flutter (Dart)</text:p>
            <text:p text:style-name="stack_lead_gray">Собственный графический движок</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="stack_cross">✘ Память: 120–160 МБ (оверхед)</text:p>
            <text:p text:style-name="stack_cross">✘ Тяжелая виртуальная машина Dart VM</text:p>
            <text:p text:style-name="stack_cross">✘ Микрофризы из-за Garbage Collector</text:p>
            <text:p text:style-name="stack_cross">✘ Ненативные эмулированные виджеты</text:p>
            <text:p text:style-name="stack_cross">✘ Большой вес бинарного файла (+40 МБ)</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Колонка 3: React Native / WebView -->
        <draw:rect svg:x="18.7cm" svg:y="3.2cm" svg:width="7.8cm" svg:height="10.8cm" draw:style-name="card_competitor">
          <draw:text-box>
            <text:p text:style-name="stack_badge_gray">ГИБРИДНЫЙ СТЕК</text:p>
            <text:p text:style-name="stack_h2">React Native / Web</text:p>
            <text:p text:style-name="stack_lead_gray">JavaScript Bridge / WebView</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="stack_cross">✘ Память: 150–220 МБ</text:p>
            <text:p text:style-name="stack_cross">✘ Узкое горлышко JS Bridge</text:p>
            <text:p text:style-name="stack_cross">✘ Задержки отклика (инпут-лаг)</text:p>
            <text:p text:style-name="stack_cross">✘ Высокий расход заряда батареи</text:p>
            <text:p text:style-name="stack_cross">✘ Риски безопасности скриптов</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 5: АРХИТЕКТУРА ПРОГРАММНОГО КОМПЛЕКСА
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide5" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">АРХИТЕКТУРА ПРОГРАММНОГО КОМПЛЕКСА</text:p>
            <text:p text:style-name="slide_subtitle">Чистая архитектура (Clean Architecture), Cargo Workspace и реактивные сигналы</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- Изображение Архитектуры (Рисунок 1) -->
        <draw:frame svg:x="1.5cm" svg:y="3.2cm" svg:width="15.5cm" svg:height="10.8cm">
          <draw:image>
            <office:binary-data>{img_arch_b64}</office:binary-data>
          </draw:image>
        </draw:frame>

        <!-- Пояснения справа -->
        <draw:rect svg:x="17.5cm" svg:y="3.2cm" svg:width="9cm" svg:height="10.8cm" draw:style-name="card_plain">
          <draw:text-box>
            <text:p text:style-name="arch_item_h">Сетевой крейт eschool-api</text:p>
            <text:p text:style-name="arch_item_p">Полностью изолирован от интерфейса. Реализует криптографию OAuth 2.0 PKCE и отказоустойчивые HTTP-запросы.</text:p>
            <text:p text:style-name="p_space"/>

            <text:p text:style-name="arch_item_h">Реактивное состояние AppState</text:p>
            <text:p text:style-name="arch_item_p">Построено на сигналах (Signals). При изменении отметок перерисовывается только конкретная ячейка, а не экран целиком.</text:p>
            <text:p text:style-name="p_space"/>

            <text:p text:style-name="arch_item_h">Слой нативных адаптеров</text:p>
            <text:p text:style-name="arch_item_p">Низкоуровневые мосты к Objective-C Runtime для аппаратного управления клавиатурой, тактильным откликом и Face ID.</text:p>
            <text:p text:style-name="p_space"/>

            <text:p text:style-name="arch_item_h">Десктопные платформы</text:p>
            <text:p text:style-name="arch_item_p">Оконная геометрия в Day.toml обеспечивает 100% работу на Windows, macOS и Linux.</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 6: АППАРАТНАЯ БЕЗОПАСНОСТЬ
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide6" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">КРОССПЛАТФОРМЕННАЯ АППАРАТНАЯ БЕЗОПАСНОСТЬ</text:p>
            <text:p text:style-name="slide_subtitle">Изоляция секретов в крипточипах (Hardware Root of Trust) и биометрический контроль</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- 4 Платформы -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="4.2cm" draw:style-name="card_sec">
          <draw:text-box>
            <text:p text:style-name="sec_os">Apple iOS / iPadOS / macOS</text:p>
            <text:p text:style-name="sec_sub">Служба Keychain Services + чип Secure Enclave (SEP)</text:p>
            <text:p text:style-name="sec_p">• Изолированный процессор со своей sepOS и TRNG</text:p>
            <text:p text:style-name="sec_p">• Неизвлекаемый аппаратный ключ UID (AES-256)</text:p>
            <text:p text:style-name="sec_p">• Защита kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="4.2cm" draw:style-name="card_sec">
          <draw:text-box>
            <text:p text:style-name="sec_os">Google Android</text:p>
            <text:p text:style-name="sec_sub">Android Keystore + TEE (TrustZone) / StrongBox</text:p>
            <text:p text:style-name="sec_p">• Аппаратные HSM-модули (Titan M2, Knox Vault)</text:p>
            <text:p text:style-name="sec_p">• Операции шифрования не покидают защищенную зону</text:p>
            <text:p text:style-name="sec_p">• Привязка к биометрии: setUserAuthenticationRequired</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="1.5cm" svg:y="7.8cm" svg:width="12cm" svg:height="4.2cm" draw:style-name="card_sec">
          <draw:text-box>
            <text:p text:style-name="sec_os">Microsoft Windows</text:p>
            <text:p text:style-name="sec_sub">Credential Manager + DPAPI + TPM 2.0</text:p>
            <text:p text:style-name="sec_p">• Аппаратный криптопроцессор TPM 2.0</text:p>
            <text:p text:style-name="sec_p">• Виртуализация безопасности VBS (Virtualization Security)</text:p>
            <text:p text:style-name="sec_p">• Биометрическая аутентификация Windows Hello</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="7.8cm" svg:width="12cm" svg:height="4.2cm" draw:style-name="card_sec">
          <draw:text-box>
            <text:p text:style-name="sec_os">Linux (FreeDesktop) &amp; HarmonyOS</text:p>
            <text:p text:style-name="sec_sub">Secret Service API (D-Bus) + HUKS</text:p>
            <text:p text:style-name="sec_p">• Системные службы gnome-keyring и KWallet</text:p>
            <text:p text:style-name="sec_p">• Аппаратная разблокировка через модуль PAM / fprintd</text:p>
            <text:p text:style-name="sec_p">• HarmonyOS: доверенная среда HUKS TEE</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Нижний блок биометрии -->
        <draw:rect svg:x="1.5cm" svg:y="12.4cm" svg:width="25cm" svg:height="2.2cm" draw:style-name="card_bio">
          <draw:text-box>
            <text:p text:style-name="bio_h">МНОГОУРОВНЕВЫЙ КОНТРОЛЬ ДОСТУПА В ПРИЛОЖЕНИИ:</text:p>
            <text:p text:style-name="bio_p">Вход по биометрии (Face ID / Touch ID) + 4-значный посоленный PIN-код + сторожевой таймер фоновой автоблокировки через 5 минут простоя.</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 7: ИНЖЕНЕРНЫЕ РЕШЕНИЯ И UX
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide7" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">ИНЖЕНЕРНЫЕ РЕШЕНИЯ И ОПТИМИЗАЦИЯ ПОЛЬЗОВАТЕЛЬСКОГО ОПЫТА (UX)</text:p>
            <text:p text:style-name="slide_subtitle">Внимание к деталям: микроанимации, плавность 120 Гц и системная интеграция</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- 3 Фичи -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="8cm" svg:height="7.5cm" draw:style-name="card_feature">
          <draw:text-box>
            <text:p text:style-name="feat_num">120 FPS</text:p>
            <text:p text:style-name="feat_h">Клавиатурный фолловер</text:p>
            <text:p text:style-name="feat_p">Прямой перехват уведомлений UIKeyboardWillShowNotification. Высота и кривая системной анимации считываются нативно через Objective-C Runtime, устраняя рывки и перекрытие полей ввода.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="10.0cm" svg:y="3.2cm" svg:width="8cm" svg:height="7.5cm" draw:style-name="card_feature">
          <draw:text-box>
            <text:p text:style-name="feat_num">LOTTIE</text:p>
            <text:p text:style-name="feat_h">Интерактивный персонаж</text:p>
            <text:p text:style-name="feat_p">Векторная анимация утки Telegram. Персонаж следит за курсором при вводе логина и закрывает лапками глаза при переходе к паролю. Легковесный нативный рендеринг Metal.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="18.5cm" svg:y="3.2cm" svg:width="8cm" svg:height="7.5cm" draw:style-name="card_feature">
          <draw:text-box>
            <text:p text:style-name="feat_num">ISLAND</text:p>
            <text:p text:style-name="feat_h">Капсула Dynamic Island</text:p>
            <text:p text:style-name="feat_p">Плавающий индикатор сетевой активности: плавно расширяется при синхронизации и информирует о статусе «Офлайн». По тапу открывает шторку системной сетевой диагностики.</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Нижнее изображение экранных форм (Рисунок П.1.2) -->
        <draw:frame svg:x="1.5cm" svg:y="11.0cm" svg:width="25cm" svg:height="3.6cm">
          <draw:image>
            <office:binary-data>{img_diary_b64}</office:binary-data>
          </draw:image>
        </draw:frame>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 8: НАДЕЖНОСТЬ И OFFLINE-FIRST
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide8" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">НАДЕЖНОСТЬ: АРХИТЕКТУРА OFFLINE-FIRST</text:p>
            <text:p text:style-name="slide_subtitle">Бесперебойный доступ к данным при падениях серверов и отсутствии сети</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- Сравнение 2 Подходов -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="8.2cm" draw:style-name="card_red_border">
          <draw:text-box>
            <text:p text:style-name="comp_badge_red">ОФИЦИАЛЬНАЯ ВЕБ-ВЕРСИЯ</text:p>
            <text:p text:style-name="comp_h">Блокировка доступа</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="comp_cross">✘ При падении сервера e-schools: ошибка 502/503</text:p>
            <text:p text:style-name="comp_cross">✘ Кеш в браузере отсутствует — экран блокируется</text:p>
            <text:p text:style-name="comp_cross">✘ Невозможно посмотреть даже расписание на завтра</text:p>
            <text:p text:style-name="comp_cross">✘ При сбоях интернета оценки исчезают</text:p>
            <text:p text:style-name="comp_cross">✘ Сессия сбрасывается, требуя ручной ввод пароля</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="3.2cm" svg:width="12cm" svg:height="8.2cm" draw:style-name="card_green_border">
          <draw:text-box>
            <text:p text:style-name="comp_badge_green">ПРИЛОЖЕНИЕ «OPENSCHOOL»</text:p>
            <text:p text:style-name="comp_h">100% Автономность</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="comp_check">✔ Мгновенный переход на локальный кеш при сбое</text:p>
            <text:p text:style-name="comp_check">✔ Все недели, четвертные отметки и ДЗ сохранены</text:p>
            <text:p text:style-name="comp_check">✔ Индикация в Dynamic Island: «Офлайн: данные из кеша»</text:p>
            <text:p text:style-name="comp_check">✔ Транзакционная целостность и защита от повреждений</text:p>
            <text:p text:style-name="comp_check">✔ Автономная работа даже при физическом обрыве связи</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Баннер демонстрации -->
        <draw:rect svg:x="1.5cm" svg:y="11.8cm" svg:width="25cm" svg:height="2.8cm" draw:style-name="card_demo_badge">
          <draw:text-box>
            <text:p text:style-name="demo_h">⚡ ЖИВАЯ ДЕМОНСТРАЦИЯ ЖЮРИ НА ТЕСТОВОМ ОБРАЗЦЕ:</text:p>
            <text:p text:style-name="demo_p">Тестовый смартфон Apple iPhone X с аппаратно неработающим модулем Wi-Fi/Bluetooth передан в руки жюри: устройство физически лишено выхода в интернет, но приложение мгновенно отображает полный дневник уроков и отметок из локального кеша!</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 9: ТЕСТИРОВАНИЕ НА ПАРКЕ УСТРОЙСТВ
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide9" draw:master-page-name="MasterLight">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_light"/>

        <draw:frame svg:x="1.5cm" svg:y="1cm" svg:width="25cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="slide_title">НАТУРНОЕ ТЕСТИРОВАНИЕ НА ПАРКЕ УСТРОЙСТВ</text:p>
            <text:p text:style-name="slide_subtitle">Проверка надежности на смартфонах учащихся 11 классов гимназии № 21 г. Минска</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- 3 Устройства -->
        <draw:rect svg:x="1.5cm" svg:y="3.2cm" svg:width="8cm" svg:height="6.8cm" draw:style-name="card_device">
          <draw:text-box>
            <text:p text:style-name="dev_type">ФЛАГМАН (iOS)</text:p>
            <text:p text:style-name="dev_name">Apple iPhone 16 Pro</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="dev_bullet">• Дисплей ProMotion 120 Гц</text:p>
            <text:p text:style-name="dev_bullet">• Интеграция Dynamic Island</text:p>
            <text:p text:style-name="dev_bullet">• Биометрия Face ID</text:p>
            <text:p text:style-name="dev_bullet">• Тест максимальной плавности</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="10.0cm" svg:y="3.2cm" svg:width="8cm" svg:height="6.8cm" draw:style-name="card_device">
          <draw:text-box>
            <text:p text:style-name="dev_type">ОФЛАЙН-ТЕСТ (iOS)</text:p>
            <text:p text:style-name="dev_name">Apple iPhone X</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="dev_bullet">• Аппаратно отключен Wi-Fi/BT</text:p>
            <text:p text:style-name="dev_bullet">• Чип Secure Enclave (A11 Bionic)</text:p>
            <text:p text:style-name="dev_bullet">• Проверка автономного кеша</text:p>
            <text:p text:style-name="dev_bullet">• Тест без доступа в сеть</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="18.5cm" svg:y="3.2cm" svg:width="8cm" svg:height="6.8cm" draw:style-name="card_device">
          <draw:text-box>
            <text:p text:style-name="dev_type">ANDROID / КРОСС-ПЛАТФОРМА</text:p>
            <text:p text:style-name="dev_name">Xiaomi Mi 8 Lite</text:p>
            <text:p text:style-name="p_space"/>
            <text:p text:style-name="dev_bullet">• Процессор Snapdragon 660</text:p>
            <text:p text:style-name="dev_bullet">• Нативный Material Design</text:p>
            <text:p text:style-name="dev_bullet">• Android Keystore &amp; TEE</text:p>
            <text:p text:style-name="dev_bullet">• Тест на доступном «железе»</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Нижний блок инфраструктуры -->
        <draw:rect svg:x="1.5cm" svg:y="10.4cm" svg:width="25cm" svg:height="4.2cm" draw:style-name="card_infra">
          <draw:text-box>
            <text:p text:style-name="infra_h">ИНФРАСТРУКТУРА ЛОКАЛЬНОГО ТЕСТИРОВАНИЯ И OPEN SOURCE:</text:p>
            <text:p text:style-name="infra_p">• Сборка в нативный iOS-пакет (.ipa) и дистрибуция через среду LiveContainer без джейлбрейка.</text:p>
            <text:p text:style-name="infra_p">• Разработан легковесный сервер мгновенного развертывания сборок по Wi-Fi (scripts/serve.sh).</text:p>
            <text:p text:style-name="infra_p">• Проведено более 50 итераций тестирования (версия 0.1.0, стабильный build 52/73).</text:p>
            <text:p text:style-name="infra_p">• Открытый исходный код на GitHub (репозиторий eschool-app): абсолютная прозрачность для аудита безопасности.</text:p>
          </draw:text-box>
        </draw:rect>
      </draw:page>
    ''')

    # =========================================================================
    # СЛАЙД 10: ЗАКЛЮЧЕНИЕ, ЭКОСИСТЕМА И РОАДМАП
    # =========================================================================
    slides.append(f'''
      <draw:page draw:name="Slide10" draw:master-page-name="MasterDark">
        <draw:rect svg:x="0cm" svg:y="0cm" svg:width="28cm" svg:height="15.75cm" draw:style-name="bg_dark"/>

        <draw:frame svg:x="2cm" svg:y="1.2cm" svg:width="24cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="title_h1_white">ИТОГИ ИССЛЕДОВАНИЯ И ПЛАНЫ РАЗВИТИЯ</text:p>
            <text:p text:style-name="title_sub_cyan">Переход от мобильного клиента к комплексной государственной цифровой экосистеме</text:p>
          </draw:text-box>
        </draw:frame>

        <!-- 4 Плана развития -->
        <draw:rect svg:x="2cm" svg:y="3.4cm" svg:width="11.5cm" svg:height="4.2cm" draw:style-name="card_dark">
          <draw:text-box>
            <text:p text:style-name="plan_badge">ПЛАТФОРМА</text:p>
            <text:p text:style-name="plan_h">Нативные Десктоп-версии</text:p>
            <text:p text:style-name="plan_p">Сборки для Windows (XAML), macOS (AppKit) и отечественных ОС Linux (Astra, РЕД ОС, Alt Linux) с аппаратной защитой TPM 2.0.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="3.4cm" svg:width="11.5cm" svg:height="4.2cm" draw:style-name="card_dark">
          <draw:text-box>
            <text:p text:style-name="plan_badge">ПЕДАГОГИКА</text:p>
            <text:p text:style-name="plan_h">Учительская версия (Журнал)</text:p>
            <text:p text:style-name="plan_p">Электронный классный журнал с выставлением отметок прямо на уроке в 100% офлайне — защита от потери оценок при падении серверов.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="2cm" svg:y="8.0cm" svg:width="11.5cm" svg:height="4.2cm" draw:style-name="card_dark">
          <draw:text-box>
            <text:p text:style-name="plan_badge">СЕМЬЯ</text:p>
            <text:p text:style-name="plan_h">Родительский профиль</text:p>
            <text:p text:style-name="plan_p">Мониторинг успеваемости нескольких детей в едином окне без смены учетных записей и функция электронной росписи в дневнике.</text:p>
          </draw:text-box>
        </draw:rect>

        <draw:rect svg:x="14.5cm" svg:y="8.0cm" svg:width="11.5cm" svg:height="4.2cm" draw:style-name="card_dark">
          <draw:text-box>
            <text:p text:style-name="plan_badge">ДИСТРИБУЦИЯ</text:p>
            <text:p text:style-name="plan_h">Релиз в App Store и RuStore</text:p>
            <text:p text:style-name="plan_p">Прохождение официальной модерации и публикация в каталогах мобильных приложений для свободного скачивания всеми учащимися.</text:p>
          </draw:text-box>
        </draw:rect>

        <!-- Финальная благодарность -->
        <draw:frame svg:x="2cm" svg:y="12.8cm" svg:width="24cm" svg:height="1.8cm">
          <draw:text-box>
            <text:p text:style-name="final_thanks">СПАСИБО ЗА ВНИМАНИЕ! ГОТОВ ОТВЕТИТЬ НА ВАШИ ВОПРОСЫ.</text:p>
            <text:p text:style-name="final_git">Репозиторий проекта: github.com/kayzer4/eschool-app • Версия 0.1.0</text:p>
          </draw:text-box>
        </draw:frame>
      </draw:page>
    ''')

    all_slides_xml = "\n".join(slides)

    fodp_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
                 xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
                 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
                 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
                 xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
                 xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0"
                 office:mimetype="application/vnd.oasis.opendocument.presentation"
                 office:version="1.2">
  <office:automatic-styles>
    <!-- 16:9 Landscape Page Layout -->
    <style:page-layout style:name="Layout16x9">
      <style:page-layout-properties fo:page-width="28cm" fo:page-height="15.75cm"
                                    fo:margin-top="0cm" fo:margin-bottom="0cm"
                                    fo:margin-left="0cm" fo:margin-right="0cm"
                                    style:print-orientation="landscape"/>
    </style:page-layout>

    <style:master-page style:name="MasterDark" style:page-layout-name="Layout16x9"/>
    <style:master-page style:name="MasterLight" style:page-layout-name="Layout16x9"/>

    <!-- Фоновые стили -->
    <style:style style:name="bg_dark" style:family="drawing-page">
      <style:drawing-page-properties draw:fill="solid" draw:fill-color="#0f172a"/>
    </style:style>
    <style:style style:name="bg_dark" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#0f172a" draw:stroke="none"/>
    </style:style>

    <style:style style:name="bg_light" style:family="drawing-page">
      <style:drawing-page-properties draw:fill="solid" draw:fill-color="#f8fafc"/>
    </style:style>
    <style:style style:name="bg_light" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f8fafc" draw:stroke="none"/>
    </style:style>

    <!-- Стили карточек -->
    <style:style style:name="card_dark" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#1e293b" draw:stroke="none" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_dark_border" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#1e293b" draw:stroke="solid" svg:stroke-color="#38bdf8" svg:stroke-width="0.05cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="badge_cyan" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#0284c7" draw:stroke="none" svg:rx="0.2cm" svg:ry="0.2cm"/>
    </style:style>

    <style:style style:name="card_red" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#fef2f2" draw:stroke="solid" svg:stroke-color="#f87171" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_amber" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#fffbeb" draw:stroke="solid" svg:stroke-color="#fbbf24" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_blue_accent" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#0284c7" draw:stroke="none" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_goal" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f0f9ff" draw:stroke="solid" svg:stroke-color="#0ea5e9" svg:stroke-width="0.05cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_task" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#e2e8f0" svg:stroke-width="0.03cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_winner" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f0fdf4" draw:stroke="solid" svg:stroke-color="#22c55e" svg:stroke-width="0.06cm" svg:rx="0.4cm" svg:ry="0.4cm"/>
    </style:style>
    <style:style style:name="card_competitor" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#e2e8f0" svg:stroke-width="0.03cm" svg:rx="0.4cm" svg:ry="0.4cm"/>
    </style:style>
    <style:style style:name="card_plain" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#cbd5e1" svg:stroke-width="0.03cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_sec" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#0284c7" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_bio" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f8fafc" draw:stroke="solid" svg:stroke-color="#3b82f6" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_feature" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#e2e8f0" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_red_border" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#ef4444" svg:stroke-width="0.05cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_green_border" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f0fdf4" draw:stroke="solid" svg:stroke-color="#16a34a" svg:stroke-width="0.06cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_demo_badge" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#eff6ff" draw:stroke="solid" svg:stroke-color="#2563eb" svg:stroke-width="0.05cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <style:style style:name="card_device" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#ffffff" draw:stroke="solid" svg:stroke-color="#0ea5e9" svg:stroke-width="0.04cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>
    <style:style style:name="card_infra" style:family="graphic">
      <style:graphic-properties draw:fill="solid" draw:fill-color="#f8fafc" draw:stroke="solid" svg:stroke-color="#cbd5e1" svg:stroke-width="0.03cm" svg:rx="0.3cm" svg:ry="0.3cm"/>
    </style:style>

    <!-- Стили текста -->
    <style:style style:name="badge_text" style:family="paragraph">
      <style:paragraph-properties fo:text-align="center" fo:margin-top="0.15cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="title_h1" style:family="paragraph">
      <style:text-properties fo:font-size="20pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="title_sub" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm"/>
      <style:text-properties fo:font-size="14pt" fo:font-weight="bold" fo:color="#38bdf8" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="card_h_cyan" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#38bdf8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_text_white" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="14pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_text_muted" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:color="#94a3b8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_h_emerald" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#34d399" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="list_bullet_white" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.1cm"/>
      <style:text-properties fo:font-size="11pt" fo:color="#f1f5f9" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="footer_dark" style:family="paragraph">
      <style:paragraph-properties fo:text-align="center"/>
      <style:text-properties fo:font-size="11pt" fo:color="#64748b" style:font-name="Arial"/>
    </style:style>

    <!-- Заголовки слайдов -->
    <style:style style:name="slide_title" style:family="paragraph">
      <style:text-properties fo:font-size="18pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="slide_subtitle" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm"/>
      <style:text-properties fo:font-size="12pt" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>

    <!-- Карточки контента -->
    <style:style style:name="card_h_red" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="13pt" fo:font-weight="bold" fo:color="#b91c1c" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_h_amber" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="13pt" fo:font-weight="bold" fo:color="#b45309" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:color="#334155" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="banner_text" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.4cm" fo:text-align="center"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="card_h_blue" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="card_goal_text" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="task_num" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="16pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="task_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.05cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="task_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#475569" style:font-name="Arial"/>
    </style:style>

    <!-- Стек стилей -->
    <style:style style:name="stack_badge_green" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#16a34a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_h1" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="18pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_lead" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_check" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.15cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#15803d" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_note" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#64748b" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="stack_badge_gray" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#64748b" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_h2" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="16pt" fo:font-weight="bold" fo:color="#334155" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_lead_gray" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#64748b" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="stack_cross" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.15cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#64748b" style:font-name="Arial"/>
    </style:style>

    <!-- Архитектурные стили -->
    <style:style style:name="arch_item_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="arch_item_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#334155" style:font-name="Arial"/>
    </style:style>

    <!-- Стили безопасности -->
    <style:style style:name="sec_os" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.25cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="13pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="sec_sub" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="sec_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.08cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#475569" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="bio_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#1e40af" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="bio_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>

    <!-- Стили фич -->
    <style:style style:name="feat_num" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="16pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="feat_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.05cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="13pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="feat_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.15cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#475569" style:font-name="Arial"/>
    </style:style>

    <!-- Стили сравнения -->
    <style:style style:name="comp_badge_red" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#dc2626" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="comp_badge_green" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#16a34a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="comp_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="16pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="comp_cross" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.15cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:color="#dc2626" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="comp_check" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.15cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#15803d" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="demo_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.25cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#1d4ed8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="demo_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm" fo:margin-left="0.4cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="11pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>

    <!-- Стили девайсов -->
    <style:style style:name="dev_type" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.3cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="dev_name" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="14pt" fo:font-weight="bold" fo:color="#0f172a" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="dev_bullet" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.1cm"/>
      <style:text-properties fo:font-size="11pt" fo:color="#334155" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="infra_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.25cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="12pt" fo:font-weight="bold" fo:color="#0284c7" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="infra_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.08cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#334155" style:font-name="Arial"/>
    </style:style>

    <!-- Слайд 10 стили -->
    <style:style style:name="title_h1_white" style:family="paragraph">
      <style:text-properties fo:font-size="20pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="title_sub_cyan" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.15cm"/>
      <style:text-properties fo:font-size="13pt" fo:color="#38bdf8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="plan_badge" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.25cm" fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:font-weight="bold" fo:color="#38bdf8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="plan_h" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm"/>
      <style:text-properties fo:font-size="13pt" fo:font-weight="bold" fo:color="#ffffff" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="plan_p" style:family="paragraph">
      <style:paragraph-properties fo:margin-left="0.4cm" fo:margin-top="0.1cm" fo:margin-right="0.4cm"/>
      <style:text-properties fo:font-size="10pt" fo:color="#cbd5e1" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="final_thanks" style:family="paragraph">
      <style:paragraph-properties fo:text-align="center"/>
      <style:text-properties fo:font-size="16pt" fo:font-weight="bold" fo:color="#38bdf8" style:font-name="Arial"/>
    </style:style>
    <style:style style:name="final_git" style:family="paragraph">
      <style:paragraph-properties fo:text-align="center" fo:margin-top="0.15cm"/>
      <style:text-properties fo:font-size="12pt" fo:color="#94a3b8" style:font-name="Arial"/>
    </style:style>

    <style:style style:name="p_space" style:family="paragraph">
      <style:paragraph-properties fo:margin-top="0.1cm"/>
    </style:style>
  </office:automatic-styles>

  <office:master-styles>
    <style:master-page style:name="MasterDark" style:page-layout-name="Layout16x9"/>
    <style:master-page style:name="MasterLight" style:page-layout-name="Layout16x9"/>
  </office:master-styles>

  <office:body>
    <office:presentation>
{all_slides_xml}
    </office:presentation>
  </office:body>
</office:document>'''

    print(f"Writing temporary Flat ODF Presentation to {TEMP_FODP}...")
    with open(TEMP_FODP, "w", encoding="utf-8") as f:
        f.write(fodp_content)

    # 1. Convert to PPTX
    print(f"Converting to PowerPoint PPTX: {OUTPUT_PPTX}...")
    subprocess.check_call(["libreoffice", "--headless", "--convert-to", "pptx", TEMP_FODP, "--outdir", OUTPUT_DIR])
    base_name = os.path.splitext(os.path.basename(TEMP_FODP))[0]
    generated_pptx = os.path.join(OUTPUT_DIR, f"{base_name}.pptx")
    if os.path.exists(OUTPUT_PPTX):
        os.remove(OUTPUT_PPTX)
    shutil.move(generated_pptx, OUTPUT_PPTX)
    print(f"Successfully created PPTX: {OUTPUT_PPTX} ({os.path.getsize(OUTPUT_PPTX)} bytes)")

    # 2. Convert to PDF
    print(f"Converting to Presentation PDF: {OUTPUT_PDF}...")
    subprocess.check_call(["libreoffice", "--headless", "--convert-to", "pdf", OUTPUT_PPTX, "--outdir", OUTPUT_DIR])
    print(f"Successfully created PDF: {OUTPUT_PDF} ({os.path.getsize(OUTPUT_PDF)} bytes)")

if __name__ == "__main__":
    import shutil
    build_presentation()
