use crate::device;
use crate::device::OutboundFrame;
use axum::extract::State;
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

// Define the commands the server can send to the main egui thread
#[derive(Debug, Clone)]
pub enum ApiCommand {
    Start(OutboundFrame),
    Continue(OutboundFrame),
    Stop,
}

#[derive(Deserialize)]
struct ConstantCurrentRequest {
    pub discharge_current: f32,
    pub discharge_cutoff_voltage: f32,
    pub discharge_time_enabled: bool,
    pub discharge_time: u16,
}

#[derive(Deserialize)]
struct ConstantPowerRequest {
    pub discharge_watts: u16,
    pub discharge_cutoff_voltage: f32,
    pub discharge_time_enabled: bool,
    pub discharge_time: u16,
}

#[derive(Deserialize)]
struct ConstantVoltageRequest {
    pub charge_current: f32,
    pub charge_voltage: f32,
    pub charge_cutoff_current: f32,
}

pub type ApiSender = Sender<ApiCommand>;

pub struct APIServer {
    router: Router,
    listener: tokio::net::TcpListener,
}

impl APIServer {
    pub async fn new(tx: ApiSender) -> Result<Self, Box<dyn std::error::Error>> {
        let router = Router::new()
            // --- Constant Current (CC) Control ---
            .route("/discharge/constant-current/start", post(Self::start_cc))
            .route("/discharge/constant-current/stop", post(Self::stop_cc))
            .route(
                "/discharge/constant-current/continue",
                post(Self::continue_cc),
            )
            // --- Constant Power (CP) Control ---
            .route("/discharge/constant-power/start", post(Self::start_cp))
            .route("/discharge/constant-power/stop", post(Self::stop_cp))
            .route(
                "/discharge/constant-power/continue",
                post(Self::continue_cp),
            )
            // --- Constant Voltage (CV) Charge Control ---
            .route("/charge/constant-voltage/start", post(Self::start_cv))
            .route("/charge/constant-voltage/stop", post(Self::stop_cv))
            .route("/charge/constant-voltage/continue", post(Self::continue_cv))
            .with_state(tx); // Pass the channel sender as state instead of SharedSession

        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        Ok(Self { router, listener })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("API server is running on 127.0.0.1:8080");
        axum::serve(self.listener, self.router).await?;
        Ok(())
    }

    // --- Handlers sending commands through the channel ---

    async fn start_cc(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantCurrentRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::StartConstantCurrentDischarge(
            (payload.discharge_current * 1000.0) as u16,
            (payload.discharge_cutoff_voltage * 1000.0) as u16,
            if payload.discharge_time_enabled {
                payload.discharge_time
            } else {
                device::MIN_CUTOFF_TIME_MIN
            },
        );
        drop(tx.send(ApiCommand::Start(frame)).await);
        StatusCode::OK
    }

    async fn stop_cc(State(tx): State<ApiSender>) -> impl IntoResponse {
        drop(tx.send(ApiCommand::Stop).await);
        StatusCode::OK
    }

    async fn continue_cc(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantCurrentRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::ContinueConstantCurrentDischarge(
            (payload.discharge_current * 1000.0) as u16,
            (payload.discharge_cutoff_voltage * 1000.0) as u16,
            if payload.discharge_time_enabled {
                payload.discharge_time
            } else {
                device::MIN_CUTOFF_TIME_MIN
            },
        );
        drop(tx.send(ApiCommand::Continue(frame)).await);
        StatusCode::OK
    }

    async fn start_cp(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantPowerRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::StartConstantPowerDischarge(
            payload.discharge_watts,
            (payload.discharge_cutoff_voltage * 1000.0) as u16,
            if payload.discharge_time_enabled {
                payload.discharge_time
            } else {
                device::MIN_CUTOFF_TIME_MIN
            },
        );
        drop(tx.send(ApiCommand::Start(frame)).await);
        StatusCode::OK
    }

    async fn stop_cp(State(tx): State<ApiSender>) -> impl IntoResponse {
        drop(tx.send(ApiCommand::Stop).await);
        StatusCode::OK
    }

    async fn continue_cp(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantPowerRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::ContinueConstantPowerDischarge(
            payload.discharge_watts,
            (payload.discharge_cutoff_voltage * 1000.0) as u16,
            if payload.discharge_time_enabled {
                payload.discharge_time
            } else {
                device::MIN_CUTOFF_TIME_MIN
            },
        );
        drop(tx.send(ApiCommand::Continue(frame)).await);
        StatusCode::OK
    }

    async fn start_cv(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantVoltageRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::StartConstantVoltageCharge(
            (payload.charge_current * 1000.0) as u16,
            (payload.charge_voltage * 1000.0) as u16,
            (payload.charge_cutoff_current * 1000.0) as u16,
        );
        drop(tx.send(ApiCommand::Start(frame)).await);
        StatusCode::OK
    }

    async fn stop_cv(State(tx): State<ApiSender>) -> impl IntoResponse {
        drop(tx.send(ApiCommand::Stop).await);
        StatusCode::OK
    }

    async fn continue_cv(
        State(tx): State<ApiSender>,
        Json(payload): Json<ConstantVoltageRequest>,
    ) -> impl IntoResponse {
        let frame = OutboundFrame::ContinueConstantVoltageCharge(
            (payload.charge_current * 1000.0) as u16,
            (payload.charge_voltage * 1000.0) as u16,
            (payload.charge_cutoff_current * 1000.0) as u16,
        );
        drop(tx.send(ApiCommand::Continue(frame)).await);
        StatusCode::OK
    }
}
