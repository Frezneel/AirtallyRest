// Handler untuk decode barcode IATA
pub async fn decode_barcode(
    State(pool): State<PgPool>,
    Json(payload): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodedBarcode>>), AppError> {
    payload.validate()?;
    let decoded = database::decode_barcode_iata(&pool, payload).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Barcode decoded successfully".to_string()),
        data: Some(decoded),
        total: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk mendapatkan semua decoded barcodes
pub async fn get_decoded_barcodes(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<DecodedBarcode>>>, AppError> {
    let decoded_list = database::get_all_decoded_barcodes(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(decoded_list),
        total: None,
    };
    Ok(Json(response))
}