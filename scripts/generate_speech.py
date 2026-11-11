#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Генератор текста доклада и защитной записки к исследовательской работе.
Автор: Кусков Станислав Андреевич, 11 «Б» класс, ГУО «Гимназия № 21 г. Минска».
Выходной файл: /home/kayzer/Downloads/Доклад_к_защите_Кусков_OpenSchool.docx (+ .pdf)
"""

import os
import shutil
import subprocess
import zipfile

OUTPUT_DOCX = "/home/kayzer/Downloads/Доклад_к_защите_Кусков_OpenSchool.docx"
OUTPUT_PDF = "/home/kayzer/Downloads/Доклад_к_защите_Кусков_OpenSchool.pdf"
OUTPUT_DIR = "/home/kayzer/Downloads"
TEMPLATE_PATH = "/home/kayzer/Downloads/Тезисы_eschool_app.docx"

def escape_xml(text):
    return (text.replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace('"', "&quot;")
                .replace("'", "&apos;"))

class DocxBuilder:
    def __init__(self):
        self.paragraphs = []

    def p_empty(self, count=1):
        for _ in range(count):
            self.paragraphs.append('''
<w:p>
  <w:pPr><w:spacing w:line="288" w:lineRule="auto" w:before="0" w:after="0"/></w:pPr>
</w:p>''')

    def page_break(self):
        self.paragraphs.append('''
<w:p>
  <w:r><w:br w:type="page"/></w:r>
</w:p>''')

    def p_title(self, text):
        t = escape_xml(text)
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="center"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="120" w:after="120"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:sz w:val="32"/>
      <w:szCs w:val="32"/>
    </w:rPr>
    <w:t>{t}</w:t>
  </w:r>
</w:p>''')

    def p_subtitle(self, text):
        t = escape_xml(text)
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="center"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="60" w:after="160"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:i/>
      <w:sz w:val="26"/>
      <w:szCs w:val="26"/>
    </w:rPr>
    <w:t>{t}</w:t>
  </w:r>
</w:p>''')

    def p_h1(self, text):
        t = escape_xml(text)
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="center"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="240" w:after="120"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:sz w:val="28"/>
      <w:szCs w:val="28"/>
    </w:rPr>
    <w:t>{t}</w:t>
  </w:r>
</w:p>''')

    def p_slide_badge(self, slide_num, title, duration):
        s_text = f"СЛАЙД {slide_num}. {title.upper()} [{duration}]"
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="left"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="200" w:after="60"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:color w:val="003366"/>
      <w:sz w:val="26"/>
      <w:szCs w:val="26"/>
    </w:rPr>
    <w:t>▶ {escape_xml(s_text)}</w:t>
  </w:r>
</w:p>''')

    def p_action_note(self, text):
        t = escape_xml(text)
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:ind w:left="720"/>
    <w:spacing w:line="240" w:lineRule="auto" w:before="40" w:after="40"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:i/>
      <w:color w:val="707070"/>
      <w:sz w:val="22"/>
      <w:szCs w:val="22"/>
    </w:rPr>
    <w:t>⚡ Действие / визуал на слайде: {t}</w:t>
  </w:r>
</w:p>''')

    def p_speech(self, text):
        t = escape_xml(text)
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="both"/>
    <w:ind w:firstLine="709"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="40" w:after="80"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:sz w:val="26"/>
      <w:szCs w:val="26"/>
    </w:rPr>
    <w:t>{t}</w:t>
  </w:r>
</w:p>''')

    def p_info_box(self, label, text):
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="both"/>
    <w:ind w:firstLine="0" w:left="360" w:right="360"/>
    <w:spacing w:line="260" w:lineRule="auto" w:before="80" w:after="80"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:sz w:val="24"/>
      <w:szCs w:val="24"/>
    </w:rPr>
    <w:t xml:space="preserve">{escape_xml(label)}: </w:t>
  </w:r>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:sz w:val="24"/>
      <w:szCs w:val="24"/>
    </w:rPr>
    <w:t>{escape_xml(text)}</w:t>
  </w:r>
</w:p>''')

    def p_faq_q(self, num, question):
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="both"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="160" w:after="40"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:color w:val="990000"/>
      <w:sz w:val="26"/>
      <w:szCs w:val="26"/>
    </w:rPr>
    <w:t xml:space="preserve">Вопрос {num}: </w:t>
  </w:r>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:sz w:val="26"/>
      <w:szCs w:val="26"/>
    </w:rPr>
    <w:t>«{escape_xml(question)}»</w:t>
  </w:r>
</w:p>''')

    def p_faq_a(self, answer):
        self.paragraphs.append(f'''
<w:p>
  <w:pPr>
    <w:jc w:val="both"/>
    <w:ind w:firstLine="709"/>
    <w:spacing w:line="288" w:lineRule="auto" w:before="40" w:after="120"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:b/>
      <w:sz w:val="24"/>
      <w:szCs w:val="24"/>
    </w:rPr>
    <w:t xml:space="preserve">Рекомендуемый ответ: </w:t>
  </w:r>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
      <w:sz w:val="24"/>
      <w:szCs w:val="24"/>
    </w:rPr>
    <w:t>{escape_xml(answer)}</w:t>
  </w:r>
</w:p>''')

    def build_document_xml(self):
        sectPr = '''
  <w:sectPr>
    <w:footerReference w:type="default" r:id="rIdFtr"/>
    <w:pgSz w:w="11906" w:h="16838"/>
    <w:pgMar w:top="1134" w:right="850" w:bottom="1134" w:left="1417" w:header="709" w:footer="709" w:gutter="0"/>
    <w:cols w:space="708"/>
    <w:docGrid w:linePitch="360"/>
  </w:sectPr>
'''
        body_content = "\n".join(self.paragraphs) + sectPr
        return f'''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
{body_content}
  </w:body>
</w:document>'''

def build_speech_document():
    doc = DocxBuilder()

    # Заголовок
    doc.p_title("ТЕКСТ ДОКЛАДА И ЗАЩИТНАЯ ЗАПИСКА")
    doc.p_subtitle("к исследовательской работе на тему: «Разработка кроссплатформенного приложения «eschool-app» для доступа к электронному дневнику на языке Rust с использованием UI-фреймворка Day» (публичный бренд «OpenSchool»)")

    doc.p_info_box("Автор работы", "Кусков Станислав Андреевич, учащийся 11 «Б» класса ГУО «Гимназия № 21 г. Минска»")
    doc.p_info_box("Научный руководитель", "Кузнецова Ольга Михайловна, учитель информатики")
    doc.p_info_box("Регламент выступления", "7 минут на доклад + 3–5 минут на ответы на вопросы жюри")
    doc.p_info_box("Целевой темп речи", "110–120 слов в минуту (уверенно, с паузами на переключение слайдов)")

    doc.p_empty(1)

    # =========================================================================
    # РАЗДЕЛ 1: ТЕКСТ ВЫСТУПЛЕНИЯ ПО СЛАЙДАМ
    # =========================================================================
    doc.p_h1("РАЗДЕЛ I. ТЕКСТ ВЫСТУПЛЕНИЯ ПО СЛАЙДАМ (7 МИНУТ)")

    # Слайд 1
    doc.p_slide_badge(1, "Титульный слайд", "0:30")
    doc.p_action_note("На экране: тема работы, ФИО автора, научного руководителя, логотип гимназии и скриншот приложения.")
    doc.p_speech(
        "Здравствуйте, уважаемые члены жюри и участники конференции! Меня зовут Кусков Станислав, я учащийся 11 «Б» класса "
        "гимназии № 21 города Минска. Мой научный руководитель — Кузнецова Ольга Михайловна. "
        "Тема моей исследовательской работы — «Разработка кроссплатформенного приложения «eschool-app» для доступа к "
        "электронному дневнику на языке Rust с использованием UI-фреймворка Day». В качестве публичного названия для "
        "пользователей утвержден бренд «OpenSchool»."
    )

    # Слайд 2
    doc.p_slide_badge(2, "Актуальность и проблематика", "0:45")
    doc.p_action_note("На экране: скриншоты ошибок 502/503 сайта e-schools.by, плашка «Технические работы», тезисы о проблемах веб-версии.")
    doc.p_speech(
        "В Республике Беларусь единой платформой учета успеваемости является сервис schools.by (e-schools.by). Ежедневно сотни тысяч "
        "учащихся и их родителей обращаются к сервису, чтобы узнать расписание, домашние задания и баллы. "
        "Однако на сегодняшний день официальный сервис существует исключительно в виде веб-сайта в браузере. Нативного мобильного "
        "приложения в App Store и Google Play нет вовсе, равно как до нашей разработки не существовало ни одного альтернативного мобильного клиента. "
        "При этом веб-версия имеет критические недостатки: вечерние «технические работы» полностью блокируют доступ (ошибки 502/503), "
        "кеш отсутствует, из-за чего ученик не может посмотреть даже расписание на завтра; при плохой связи оценки пропадают, "
        "сессии сбрасываются, а страницы потребляют до 8 мегабайт трафика."
    )

    # Слайд 3
    doc.p_slide_badge(3, "Цель, объект и задачи проекта", "0:45")
    doc.p_action_note("На экране: цель, объект, предмет исследования и 6 ключевых задач в виде инфографики.")
    doc.p_speech(
        "Цель нашей работы — спроектировать и разработать первое полноценное, высокопроизводительное и безопасное кроссплатформенное "
        "нативное мобильное приложение «OpenSchool» («eschool-app») на языке Rust с использованием UI-фреймворка Day. "
        "Для достижения цели были решены задачи: создание независимого сетевого слоя eschool-api с протоколом OAuth 2.0 PKCE; "
        "реализация аппаратного хранения токенов сессии через аппаратные чипы безопасности; разработка реактивного интерфейса со "
        "скоростью 120 кадров в секунду; внедрение 100% офлайн-кеширования и проведение тестирования на реальных устройствах учащихся."
    )

    # Слайд 4
    doc.p_slide_badge(4, "Технологический стек: почему Rust и Day?", "0:50")
    doc.p_action_note("На экране: диаграмма сравнения потребления памяти (Rust/Day 28 МБ vs Flutter 130 МБ) и схема компиляции в системный UIKit.")
    doc.p_speech(
        "Почему мы выбрали именно Rust и Day, а не популярные Flutter или React Native? "
        "Традиционные фреймворки несут тяжелый оверхед: виртуальные машины Dart или JavaScript, сборщик мусора (GC), приводящий к "
        "микрофризам анимаций, и потребление более 120 мегабайт оперативной памяти. "
        "Язык программирования Rust обеспечивает безопасность памяти на этапе компиляции без сборщика мусора и непревзойденное быстродействие. "
        "А новейший фреймворк Day транслирует декларативный интерфейс напрямую в нативные системные виджеты операционной системы: "
        "UIKit на iOS, Material Components на Android. В итоге приложение потребляет менее 30 мегабайт памяти, запускается мгновенно "
        "и работает с бескомпромиссной аппаратной плавностью."
    )

    # Слайд 5
    doc.p_slide_badge(5, "Архитектура программного комплекса", "0:50")
    doc.p_action_note("На экране: Рисунок 1 из работы — слои Clean Architecture (eschool-api, state, view, secure, native).")
    doc.p_speech(
        "Проект спроектирован по принципам Чистой Архитектуры (Clean Architecture) в виде Cargo Workspace. "
        "Сетевой крейт eschool-api полностью изолирован и не зависит от UI: он выполняет авторизацию по защищенному протоколу "
        "OAuth 2.0 с динамической генерацией криптографического верификатора PKCE, автоматически обрабатывает сетевые сбои "
        "и сериализует данные уроков и четвертных отметок. "
        "Центральное состояние AppState построено на реактивных сигналах (Signals): при изменении данных перерисовываются только "
        "конкретные зависимые узлы дерева, без перерисовки всего экрана."
    )

    # Слайд 6
    doc.p_slide_badge(6, "Кроссплатформенная аппаратная безопасность", "1:00")
    doc.p_action_note("На экране: Таблица 2 из работы — сравнение Apple Secure Enclave, Android Keystore, Windows TPM, Linux Secret Service.")
    doc.p_speech(
        "Ключевым требованием к приложению являлась бескомпромиссная защита персональных данных учащихся. В веб-браузере токены "
        "хранятся в открытом виде в LocalStorage или Cookies. В нашем приложении реализована аппаратная изоляция ключей. "
        "На платформе iOS токены шифруются в Keychain Services с привязкой к аппаратному крипточипу Secure Enclave (SEP) с неизвлекаемым "
        "ключом UID. На Android аналогичная изоляция достигается через Android Keystore и чипы StrongBox / ARM TrustZone. "
        "На Windows задействован криптопроцессор TPM 2.0 и DPAPI, а на Linux — Secret Service D-Bus API. "
        "Дополнительно доступ к приложению защищен биометрией (Face ID / Touch ID) и локальным 4-значным PIN-кодом с алгоритмом "
        "автоматической блокировки при сворачивании."
    )

    # Слайд 7
    doc.p_slide_badge(7, "Инженерные решения и пользовательский опыт", "0:50")
    doc.p_action_note("На экране: анимация Lottie-утки, схема работы клавиатурного фолловера 120 FPS и плавающая капсула Dynamic Island.")
    doc.p_speech(
        "В ходе разработки мы решили несколько сложных инженерных задач на стыке Rust и нативных платформ. "
        "Во-первых, мы синхронизировали перемещение формы ввода с системной клавиатурой с частотой 120 FPS на экранах ProMotion: "
        "через прямой перехват низкоуровневых уведомлений UIKit длительность и кривая системной анимации передаются нативным констрейнтам. "
        "Во-вторых, встроен интерактивный индикатор Dynamic Island: плавающая капсула отображает статус сети и позволяет в один тап "
        "открыть шторку диагностики. "
        "В-третьих, экран авторизации украшен интерактивной векторной Lottie-анимацией персонажа-утки, которая следит за курсором и закрывает лапками глаза при вводе пароля."
    )

    # Слайд 8
    doc.p_slide_badge(8, "Надежность и Offline-First архитектура", "0:45")
    doc.p_action_note("На экране: скриншот работы в офлайне; действие: передача в руки жюри тестового iPhone X.")
    doc.p_speech(
        "Главное практическое преимущество нашего приложения перед официальным сайтом — честная архитектура Offline-First. "
        "Все загруженные данные — недели дневника, четвертные отметки, домашние задания и прикрепленные файлы — транзакционно сохраняются в локальный кеш. "
        "Когда сервер e-schools.by падает или закрывается на «технические работы», наше приложение мгновенно переключается на офлайн-источник. "
        "Чтобы вы могли лично в этом убедиться, я передаю уважаемому жюри тестовый iPhone X. На этом телефоне аппаратно отключен модуль "
        "Wi-Fi и Bluetooth — он физически не имеет выхода в сеть. Тем не менее, вы можете лично открыть приложение «OpenSchool» и увидеть: "
        "расписание, домашние задания и оценки доступны мгновенно из защищенного локального кеша, тогда как официальный сайт в браузере "
        "показывает лишь системную ошибку отсутствия сети."
    )

    # Слайд 9
    doc.p_slide_badge(9, "Тестирование, парк устройств и Open Source", "0:45")
    doc.p_action_note("На экране: парк тестовых устройств (iPhone 16 Pro, iPhone X, Xiaomi Mi 8 Lite), GitHub репозиторий, статистика сборок.")
    doc.p_speech(
        "Проект полностью открыт под свободной лицензией (Open Source) на GitHub в репозитории `eschool-app`. Это исключает любые подозрения "
        "в скрытом сборе паролей: любой желающий может провести независимый аудит безопасности. "
        "Приложение прошло реальное тестирование на разнообразном парке физических устройств учащихся 11-х классов нашей гимназии. "
        "На современном флагмане iPhone 16 Pro подтверждена идеальная аппаратная плавность 120 FPS и интеграция с Dynamic Island; "
        "на iPhone X отработана безопасность Keychain и Secure Enclave в экстремальном офлайне; "
        "а на Xiaomi Mi 8 Lite доказана высокая производительность и кроссплатформенная переносимость стека Rust на платформе Android. "
        "Тестирование подтвердило 100% стабильность работы, корректность подсчета среднего балла и надежность биометрии."
    )

    # Слайд 10
    doc.p_slide_badge(10, "Заключение и выводы", "0:30")
    doc.p_action_note("На экране: ключевые итоги работы, QR-код на репозиторий GitHub и слова благодарности.")
    doc.p_speech(
        "В заключение хочу отметить: нами доказана высокая эффективность стека Rust + Day для создания нативных кроссплатформенных приложений. "
        "Создан надежный, быстрый и независимый клиент «OpenSchool», решающий многолетние проблемы доступа к школьному дневнику в Беларуси. "
        "Помимо мобильных версий, архитектура проекта полностью готова к сборке под десктопные операционные системы — Windows, macOS и Linux, "
        "что позволит учащимся и родителям иметь единый быстрый доступ к расписанию и оценкам на домашних компьютерах и ноутбуках. "
        "Спасибо за внимание! Я готов ответить на ваши вопросы."
    )

    doc.page_break()

    # =========================================================================
    # РАЗДЕЛ 2: СЦЕНАРИЙ ЖИВОЙ ДЕМОНСТРАЦИИ (ДЕМО НА 3 УСТРОЙСТВАХ)
    # =========================================================================
    doc.p_h1("РАЗДЕЛ II. СЦЕНАРИЙ ЖИВОЙ ДЕМОНСТРАЦИИ НА 3 ФИЗИЧЕСКИХ УСТРОЙСТВАХ")
    doc.p_speech(
        "Наличие реальных тестовых образцов — это абсолютное конкурентное преимущество перед другими участниками. "
        "Жюри очень ценит, когда проект можно физически взять в руки и потрогать, а не просто смотреть на слайды. "
        "Рекомендуется заранее подготовить и положить перед жюри три устройства со следующими ролями:"
    )
    doc.p_info_box("Образец № 1: iPhone 16 Pro (Флагман / Референс)", 
                   "Личный смартфон докладчика. Демонстрирует максимум технологических возможностей: идеальная плавность 120 FPS на дисплее ProMotion, интерактивная капсула Dynamic Island со статусом сети, мгновенная биометрическая разблокировка по Face ID и плавная Lottie-анимация при вводе пароля.")
    doc.p_info_box("Образец № 2: iPhone X (УДАРНЫЙ ОБРАЗЕЦ — Честный офлайн)", 
                   "Смартфон с аппаратно отключенным модулем Wi-Fi и Bluetooth. Это неопровержимое доказательство честной работы Offline-First: устройство физически лишено доступа к сети Интернет. Члены жюри могут лично убедиться: при попытке открыть сайт e-schools.by в Safari браузер выдает ошибку, а приложение «OpenSchool» запускается мгновенно, показывая расписание, оценки и четвертные итоги из локального защищенного кеша.")
    doc.p_info_box("Образец № 3: Xiaomi Mi 8 Lite (Кроссплатформенность / Android)", 
                   "Android-устройство на базе доступного аппаратного обеспечения. Демонстрирует переносимость стека Rust: отсутствие зависимости от экосистемы Apple, нативный Material UI, интеграцию с Android Keystore и стабильную отзывчивость даже на бюджетном процессоре благодаря скромному потреблению памяти (менее 30 МБ).")
    doc.p_speech(
        "План живого показа перед комиссией (30–45 секунд):"
    )
    doc.p_info_box("Действие 1", "В начале слайда 8 протянуть iPhone X со словами: «Уважаемые члены жюри, на этом iPhone X аппаратно отсутствует интернет. Вы можете сами полистать расписание и убедиться в надежности нашего кеша».")
    doc.p_info_box("Действие 2", "На слайде 9 положить на стол Xiaomi Mi 8 Lite и iPhone 16 Pro со словами: «Приложение работает одинаково быстро как на флагманском iPhone 16 Pro со 120 Гц, так и на Android-смартфоне Xiaomi».")
    doc.p_info_box("Эффект", "Жюри видит не «студенческий макет», а зрелый инженерный продукт, проверенный на реальном «железе». Это практически гарантирует диплом I степени.")

    doc.page_break()

    # =========================================================================
    # РАЗДЕЛ 3: ОТВЕТЫ НА КАВЕРЗНЫЕ ВОПРОСЫ ЖЮРИ (FAQ)
    # =========================================================================
    doc.p_h1("РАЗДЕЛ III. ГОТОВЫЕ ОТВЕТЫ НА ВОПРОСЫ ЖЮРИ (FAQ)")

    # Вопрос 1
    doc.p_faq_q(1, "Легально ли ваше приложение? Есть ли у e-schools.by открытый API и не нарушаете ли вы правила сервиса?")
    doc.p_faq_a(
        "Да, наше приложение абсолютно легально. Приложение выступает в качестве альтернативного пользовательского агента (User-Agent) "
        "— точно так же, как веб-браузеры Safari, Google Chrome или Firefox. Приложение не выполняет взлом серверов или обход прав: "
        "оно отправляет те же самые стандартные HTTPS-запросы к API авторизации и получения данных, которые браузер отправляет "
        "при входе пользователя на сайт e-schools.by под своим собственным логином и паролем. "
        "Кроме того, наше приложение даже снижает нагрузку на государственные серверы за счет локального кеширования: вместо повторной "
        "загрузки тяжелых веб-страниц весом по 5 мегабайт, приложение запрашивает только компактный JSON объемом 20 килобайт или читает данные из кеша."
    )

    # Вопрос 2
    doc.p_faq_q(2, "Почему именно Rust и Day, а не проверенный временем Flutter или Swift/Kotlin?")
    doc.p_faq_a(
        "Мы сознательно не использовали Flutter или React Native, так как их архитектура принципиально неоптимальна для мобильных устройств: "
        "Flutter использует собственный графический движок Impeller/Skia и виртуальную машину Dart с тяжелым сборщиком мусора, "
        "что приводит к потреблению 120–150 МБ памяти и рывкам анимаций при сборке мусора. "
        "Написание двух отдельных приложений на Swift (для iOS) и Kotlin (для Android) привело бы к дублированию бизнес-логики и сетевого кода. "
        "Rust в связке с фреймворком Day объединяет лучшее из обоих миров: 100% общая кодовая база логики на Rust с гарантией безопасности памяти, "
        "и прямая компиляция UI в нативные компоненты операционной системы UIKit / Material с минимальным потреблением 28 МБ памяти и плавностью 120 кадров в секунду."
    )

    # Вопрос 3
    doc.p_faq_q(3, "Как доказать, что ваше приложение не ворует пароли учащихся и не отправляет их на сторонние серверы?")
    doc.p_faq_a(
        "Именно для этого наш проект выполнен полностью по модели Open Source: весь исходный код приложения и библиотеки eschool-api "
        "опубликован в открытом доступе на GitHub. Любой член жюри, учитель или специалист по безопасности может открыть код и убедиться: "
        "в приложении нет сторонних аналитических библиотек, трекеров рекламы или внешних серверов. "
        "Сетевые запросы уходят исключительно на официальные домены e-schools.by по зашифрованному протоколу HTTPS (TLS 1.3). "
        "А токены сессии сохраняются не в открытых файлах, а в аппаратном анклаве Secure Enclave под защитой крипточипа устройства."
    )

    # Вопрос 4
    doc.p_faq_q(4, "Что произойдет, если e-schools.by изменит структуру своего сайта или формата данных?")
    doc.p_faq_a(
        "Благодаря модульной архитектуре Clean Architecture, сетевой слой eschool-api полностью отделен от интерфейса. "
        "Все структуры данных строго типизированы с помощью библиотеки serde: при изменении формата ответа модуль сетевой обработки "
        "не упадет в аварийную ошибку (panic), а вернет предсказуемую ошибку десериализации и автоматически переведет приложение "
        "на чтение сохраненного локального кеша. А для обновления логики запросов достаточно выпустить микропатч библиотеки eschool-api "
        "без необходимости переписывать интерфейс приложения."
    )

    # Вопрос 5
    doc.p_faq_q(5, "В чем заключается научная новизна и практический вклад вашей работы?")
    doc.p_faq_a(
        "Практический вклад огромен: создано первое в стране независимое нативное мобильное приложение для школьников и родителей, "
        "решающее острую социальную проблему недоступности дневника в вечерние часы технических работ. "
        "Научная новизна заключается в исследовании и практической апробации новой парадигмы мобильной разработки: реактивного фреймворка Day "
        "на языке Rust с прямой трансляцией в нативные системные виджеты ОС без виртуальных машин, а также создании кроссплатформенной "
        "модели аппаратной изоляции секретов (Secure Enclave / Keystore / TPM / Secret Service) в рамках единого компилируемого Rust-кода."
    )

    # Вопрос 6
    doc.p_faq_q(6, "Каковы дальнейшие планы развития проекта?")
    doc.p_faq_a(
        "Во-первых, официальный релиз в каталоги мобильных приложений: прохождение модерации в App Store и публикация в отечественном RuStore. "
        "Во-вторых, разработка специализированной Учительской версии (электронный классный журнал): выставление отметок, фиксация пропусков "
        "и ввод тем уроков прямо в классе в 100% офлайн-режиме, что навсегда избавит учителей от потери выставленных баллов при падении сервера. "
        "В-третьих, запуск Родительской версии с одновременным мониторингом успеваемости нескольких детей в семье и электронной росписью за неделю. "
        "В-четвертых, нативные десктопные сборки для Windows, macOS и отечественных ОС на базе Linux (Astra Linux, РЕД ОС), "
        "а также интерактивные виджеты расписания на рабочий стол."
    )

    # =========================================================================
    # СБОРКА ДОКУМЕНТА
    # =========================================================================
    print(f"Generating speech document to {OUTPUT_DOCX}...")
    temp_dir = "/tmp/speech_docx_build"
    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir)

    with zipfile.ZipFile(TEMPLATE_PATH, 'r') as z_in:
        z_in.extractall(temp_dir)

    # Add footer1.xml with page numbers
    ftr_xml = '''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p>
    <w:pPr><w:jc w:val="center"/></w:pPr>
    <w:fldSimple w:instr="PAGE">
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>
          <w:sz w:val="24"/>
          <w:szCs w:val="24"/>
        </w:rPr>
        <w:t>1</w:t>
      </w:r>
    </w:fldSimple>
  </w:p>
</w:ftr>'''
    with open(os.path.join(temp_dir, "word", "footer1.xml"), "w", encoding="utf-8") as f:
        f.write(ftr_xml)

    ct_path = os.path.join(temp_dir, "[Content_Types].xml")
    with open(ct_path, "r", encoding="utf-8") as f:
        ct = f.read()
    if "footer1.xml" not in ct:
        ct = ct.replace("</Types>", '<Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/></Types>')
        with open(ct_path, "w", encoding="utf-8") as f:
            f.write(ct)

    rels_path = os.path.join(temp_dir, "word", "_rels", "document.xml.rels")
    with open(rels_path, "r", encoding="utf-8") as f:
        rels_content = f.read()
    if "rIdFtr" not in rels_content:
        rels_content = rels_content.replace("</Relationships>", '<Relationship Id="rIdFtr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/></Relationships>')
        with open(rels_path, "w", encoding="utf-8") as f:
            f.write(rels_content)

    new_doc_xml = doc.build_document_xml()
    with open(os.path.join(temp_dir, "word", "document.xml"), "w", encoding="utf-8") as f:
        f.write(new_doc_xml)

    if os.path.exists(OUTPUT_DOCX):
        os.remove(OUTPUT_DOCX)

    with zipfile.ZipFile(OUTPUT_DOCX, 'w', zipfile.ZIP_DEFLATED) as z_out:
        for foldername, subfolders, filenames in os.walk(temp_dir):
            for filename in filenames:
                filepath = os.path.join(foldername, filename)
                arcname = os.path.relpath(filepath, temp_dir)
                z_out.write(filepath, arcname)

    print(f"Successfully generated DOCX: {OUTPUT_DOCX} ({os.path.getsize(OUTPUT_DOCX)} bytes)")

    # Export to PDF via LibreOffice
    print(f"Exporting PDF via LibreOffice to {OUTPUT_DIR}...")
    subprocess.check_call(["libreoffice", "--headless", "--convert-to", "pdf", OUTPUT_DOCX, "--outdir", OUTPUT_DIR])
    print(f"Successfully exported PDF: {OUTPUT_PDF} ({os.path.getsize(OUTPUT_PDF)} bytes)")

if __name__ == "__main__":
    build_speech_document()
