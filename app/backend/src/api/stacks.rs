use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::AppState;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CurveStack {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct StackComponent {
    pub id: String,
    pub stack_id: String,
    pub position: i32,
    pub label: String,
    pub curve_id: String,
    pub weight: f32,
    pub interp_method: String,
    // joined from rate_curves
    pub curve_name: Option<String>,
    pub curve_component: Option<String>,
    pub curve_series_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComponentInput {
    pub label: String,
    pub curve_id: String,
    pub weight: Option<f32>,
    pub interp_method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStackRequest {
    pub name: String,
    pub description: Option<String>,
    pub components: Vec<ComponentInput>,
}

#[derive(Debug, Deserialize)]
pub struct ComponentsForCombination {
    pub label: String,
    pub curve_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateCombinationsRequest {
    pub name_prefix: String,
    pub description: Option<String>,
    pub components: Vec<ComponentsForCombination>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn fetch_stack_with_components(
    pool: &sqlx::PgPool,
    stack_id: &str,
) -> Result<Value, StatusCode> {
    let stack = sqlx::query_as::<_, CurveStack>(
        "SELECT id, name, description, status, created_at::TEXT FROM curve_stacks WHERE id = $1",
    )
    .bind(stack_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let components = sqlx::query_as::<_, StackComponent>(
        r#"SELECT sc.id, sc.stack_id, sc.position, sc.label, sc.curve_id, sc.weight,
                  sc.interp_method,
                  rc.name AS curve_name, rc.component AS curve_component,
                  rc.series_name AS curve_series_name
           FROM curve_stack_components sc
           LEFT JOIN rate_curves rc ON rc.id = sc.curve_id
           WHERE sc.stack_id = $1
           ORDER BY sc.position"#,
    )
    .bind(stack_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(json!({
        "id": stack.id,
        "name": stack.name,
        "description": stack.description,
        "status": stack.status,
        "created_at": stack.created_at,
        "components": components,
    }))
}

async fn insert_stack_with_components(
    pool: &sqlx::PgPool,
    name: &str,
    description: Option<&str>,
    components: &[ComponentInput],
) -> Result<String, StatusCode> {
    let stack_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO curve_stacks (id, name, description) VALUES ($1, $2, $3)",
    )
    .bind(&stack_id)
    .bind(name)
    .bind(description)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (i, comp) in components.iter().enumerate() {
        let comp_id = Uuid::new_v4().to_string();
        let interp = comp.interp_method.clone().unwrap_or_else(|| "linear".into());
        sqlx::query(
            "INSERT INTO curve_stack_components (id, stack_id, position, label, curve_id, weight, interp_method)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&comp_id)
        .bind(&stack_id)
        .bind(i as i32)
        .bind(&comp.label)
        .bind(&comp.curve_id)
        .bind(comp.weight.unwrap_or(1.0))
        .bind(&interp)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(stack_id)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_stacks(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query_as::<_, CurveStack>(
        "SELECT id, name, description, status, created_at::TEXT FROM curve_stacks ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // For each stack, count components
    let mut result = Vec::new();
    for stack in rows {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM curve_stack_components WHERE stack_id = $1",
        )
        .bind(&stack.id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        result.push(json!({
            "id": stack.id,
            "name": stack.name,
            "description": stack.description,
            "status": stack.status,
            "created_at": stack.created_at,
            "component_count": count,
        }));
    }

    Ok(Json(json!(result)))
}

pub async fn get_stack(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(fetch_stack_with_components(&state.pool, &id).await?))
}

pub async fn create_stack(
    State(state): State<AppState>,
    Json(body): Json<CreateStackRequest>,
) -> Result<Json<Value>, StatusCode> {
    let stack_id =
        insert_stack_with_components(&state.pool, &body.name, body.description.as_deref(), &body.components)
            .await?;

    Ok(Json(fetch_stack_with_components(&state.pool, &stack_id).await?))
}

pub async fn update_stack(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CreateStackRequest>,
) -> Result<Json<Value>, StatusCode> {
    sqlx::query("UPDATE curve_stacks SET name = $1, description = $2 WHERE id = $3")
        .bind(&body.name)
        .bind(&body.description)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Replace components
    sqlx::query("DELETE FROM curve_stack_components WHERE stack_id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (i, comp) in body.components.iter().enumerate() {
        let comp_id = Uuid::new_v4().to_string();
        let interp = comp.interp_method.clone().unwrap_or_else(|| "linear".into());
        sqlx::query(
            "INSERT INTO curve_stack_components (id, stack_id, position, label, curve_id, weight, interp_method)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&comp_id)
        .bind(&id)
        .bind(i as i32)
        .bind(&comp.label)
        .bind(&comp.curve_id)
        .bind(comp.weight.unwrap_or(1.0))
        .bind(&interp)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(fetch_stack_with_components(&state.pool, &id).await?))
}

pub async fn delete_stack(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM curve_stacks WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/stacks/generate-combinations
/// Generates the cartesian product of curve choices per component.
pub async fn generate_combinations(
    State(state): State<AppState>,
    Json(body): Json<GenerateCombinationsRequest>,
) -> Result<Json<Value>, StatusCode> {
    if body.components.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Build cartesian product
    let mut combos: Vec<Vec<(String, String)>> = vec![vec![]]; // (label, curve_id)
    for comp in &body.components {
        if comp.curve_ids.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        let mut next = vec![];
        for existing in &combos {
            for curve_id in &comp.curve_ids {
                let mut row = existing.clone();
                row.push((comp.label.clone(), curve_id.clone()));
                next.push(row);
            }
        }
        combos = next;
    }

    let total = combos.len();
    let mut created_ids = vec![];

    for (idx, combo) in combos.iter().enumerate() {
        let suffix = if total > 1 {
            format!(" #{}", idx + 1)
        } else {
            String::new()
        };
        let name = format!("{}{}", body.name_prefix, suffix);

        let components: Vec<ComponentInput> = combo
            .iter()
            .map(|(label, curve_id)| ComponentInput {
                label: label.clone(),
                curve_id: curve_id.clone(),
                weight: None,
                interp_method: None,
            })
            .collect();

        let stack_id =
            insert_stack_with_components(&state.pool, &name, body.description.as_deref(), &components)
                .await?;
        created_ids.push(stack_id);
    }

    Ok(Json(json!({ "created": total, "stack_ids": created_ids })))
}
