use super::client::ApiClient;
use super::models::*;

pub struct Api<'a> {
    client: &'a ApiClient,
}

impl<'a> Api<'a> {
    pub fn new(client: &'a ApiClient) -> Self {
        Self { client }
    }

    // Auth endpoints
    pub async fn get_user_info(&self) -> Result<UserInfo, reqwest::Error> {
        self.client.get("/api/v1/admin/auth/me").await
    }

    // School year endpoints
    pub async fn get_school_year(&self) -> Result<SchoolYear, reqwest::Error> {
        self.client.get("/api/v1/education/diary/school_year").await
    }

    pub async fn get_time_activities(&self) -> Result<Vec<TimeActivity>, reqwest::Error> {
        self.client.get("/api/v1/education/diary/time_activities").await
    }

    pub async fn get_week_activities(&self) -> Result<Vec<WeekActivity>, reqwest::Error> {
        self.client.get("/api/v1/education/diary/time_activities/week_activities").await
    }

    // Class endpoints
    pub async fn get_classes(&self, school_id: &str, profile_id: &str) -> Result<Vec<Class>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/students/{}/classes",
            school_id, profile_id
        )).await
    }

    pub async fn get_classes_by_date(
        &self,
        school_id: &str,
        profile_id: &str,
        from: &str,
        to: &str,
        school_period: Option<&str>,
    ) -> Result<ClassesByDate, reqwest::Error> {
        let body = serde_json::json!({
            "school_period": school_period,
            "from": from,
            "to": to,
        });
        self.client.post(
            &format!(
                "/api/v1/education/diary/schools/{}/students/{}/classes",
                school_id, profile_id
            ),
            &body,
        ).await
    }

    // Lesson endpoints
    pub async fn get_lessons(
        &self,
        school_id: &str,
        class_id: &str,
        profile_id: &str,
        week_activity_uuid: &str,
    ) -> Result<Vec<DaySchedule>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/students/{}/lessons?week_activity_uuid={}",
            school_id, class_id, profile_id, week_activity_uuid
        )).await
    }

    // Master messages endpoints
    pub async fn get_master_messages(
        &self,
        school_id: &str,
        class_id: &str,
        profile_id: &str,
        week_activity_id: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/students/{}/master_messages?week_activity_id={}",
            school_id, class_id, profile_id, week_activity_id
        )).await
    }

    // Diary signatures endpoints
    pub async fn get_diary_signatures(
        &self,
        school_id: &str,
        class_id: &str,
        profile_id: &str,
        week_activity_id: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/students/{}/diary_signatures/weekly?week_activity_id={}",
            school_id, class_id, profile_id, week_activity_id
        )).await
    }

    // Final grades endpoints
    pub async fn get_final_grades(
        &self,
        school_id: &str,
        class_id: &str,
        profile_id: &str,
    ) -> Result<FinalGrades, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/students/{}/final/whole",
            school_id, class_id, profile_id
        )).await
    }

    // Subject endpoints
    pub async fn get_subjects(
        &self,
        school_id: &str,
        class_id: &str,
        profile_id: &str,
    ) -> Result<Vec<Subject>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/students/{}/educational_subjects",
            school_id, class_id, profile_id
        )).await
    }

    pub async fn get_subjects_with_teachers(
        &self,
        school_id: &str,
        profile_id: &str,
        class_id: &str,
    ) -> Result<Vec<SubjectWithTeacher>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/students/{}/classes/{}/subjects",
            school_id, profile_id, class_id
        )).await
    }

    // Bell schedule endpoints
    pub async fn get_bell_schedule(
        &self,
        school_id: &str,
    ) -> Result<Vec<BellSchedule>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/bells/whole",
            school_id
        )).await
    }

    // Timetable endpoints
    pub async fn get_timetables(
        &self,
        school_id: &str,
        class_id: &str,
    ) -> Result<Vec<Timetable>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/diary/schools/{}/classes/{}/timetables/whole",
            school_id, class_id
        )).await
    }

    // Planning endpoints
    pub async fn get_planning_subjects(
        &self,
        class_id: &str,
    ) -> Result<Vec<PlanningSubject>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/education/planning/classes/{}/educational_subjects",
            class_id
        )).await
    }

    // Premise endpoints
    pub async fn get_premises(
        &self,
        school_id: &str,
        filter: Option<&str>,
    ) -> Result<Vec<Premise>, reqwest::Error> {
        let url = if let Some(f) = filter {
            format!(
                "/api/v1/institution/schools/{}/premises?q={}",
                school_id, urlencoding::encode(f)
            )
        } else {
            format!("/api/v1/institution/schools/{}/premises", school_id)
        };
        self.client.get(&url).await
    }

    // Meal endpoints
    pub async fn get_meal_types(&self) -> Result<Vec<MealType>, reqwest::Error> {
        self.client.get("/api/v1/meals/dictionaries/meal_types").await
    }

    pub async fn get_eating_journal(
        &self,
        request: &EatingJournalRequest,
    ) -> Result<EatingJournalResponse, reqwest::Error> {
        self.client.post("/api/v1/meals/eating_journals", request).await
    }

    pub async fn get_menu_page(
        &self,
        filter: &str,
        page: u32,
        size: u32,
    ) -> Result<PaginatedResponse<serde_json::Value>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/meals/menus/page?q={}&page={}&size={}",
            urlencoding::encode(filter), page, size
        )).await
    }

    // Payment endpoints
    pub async fn get_payments(
        &self,
        school_id: &str,
        profile_id: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<Vec<serde_json::Value>, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/meals/diary/schools/{}/students/{}/payments/billing?start_ts={}&end_ts={}",
            school_id, profile_id, start_ts, end_ts
        )).await
    }

    // Supplementary lesson endpoints
    pub async fn get_supplementary_sections(
        &self,
        school_id: &str,
        filter: Option<&str>,
        page: u32,
        size: u32,
    ) -> Result<PaginatedResponse<serde_json::Value>, reqwest::Error> {
        let url = if let Some(f) = filter {
            format!(
                "/api/v1/section/diary-supplementary/schools/{}/sections/page?q={}&page={}&size={}",
                school_id, urlencoding::encode(f), page, size
            )
        } else {
            format!(
                "/api/v1/section/diary-supplementary/schools/{}/sections/page?page={}&size={}",
                school_id, page, size
            )
        };
        self.client.get(&url).await
    }

    pub async fn get_supplementary_diary(
        &self,
        profile_id: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<SupplementaryLessonDiary, reqwest::Error> {
        self.client.get(&format!(
            "/api/v1/section/diary-supplementary/students/{}/lessons/diary?start_ts={}&end_ts={}",
            profile_id, start_ts, end_ts
        )).await
    }
}
