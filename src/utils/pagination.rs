use crate::models::PaginatedResponse;

pub const DEFAULT_PAGE: i64 = 1;
pub const DEFAULT_PER_PAGE: i64 = 20;
pub const MAX_PER_PAGE: i64 = 100;

pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
}

impl Pagination {
    pub fn new(page: Option<i64>, per_page: Option<i64>) -> Self {
        let page = page.unwrap_or(DEFAULT_PAGE).max(1);
        let mut per_page = per_page.unwrap_or(DEFAULT_PER_PAGE).max(1);
        
        // 限制每页最大数量
        if per_page > MAX_PER_PAGE {
            per_page = MAX_PER_PAGE;
        }
        
        Self { page, per_page }
    }
    
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
    
    pub fn limit(&self) -> i64 {
        self.per_page
    }
    
    pub fn paginate<T>(&self, items: Vec<T>, total: i64) -> PaginatedResponse<T> {
        let total_pages = (total as f64 / self.per_page as f64).ceil() as i64;
        
        PaginatedResponse {
            items,
            total,
            page: self.page,
            per_page: self.per_page,
            total_pages,
        }
    }
} 