use leptos::*;
use leptos::html::Div;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    pub fn background_gradient(&self) -> &'static str {
        match self {
            ToastType::Success => "linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%)",
            ToastType::Error => "linear-gradient(135deg, #f8d7da 0%, #f5c6cb 100%)",
            ToastType::Warning => "linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%)",
            ToastType::Info => "linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%)",
        }
    }

    pub fn border_color(&self) -> &'static str {
        match self {
            ToastType::Success => "#28a745",
            ToastType::Error => "#dc3545",
            ToastType::Warning => "#ff9800",
            ToastType::Info => "#2196f3",
        }
    }

    pub fn text_color(&self) -> &'static str {
        match self {
            ToastType::Success => "#155724",
            ToastType::Error => "#721c24",
            ToastType::Warning => "#f57c00",
            ToastType::Info => "#1976d2",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToastType::Success => "✓", // Tick personalizzato più elegante
            ToastType::Error => "✕",
            ToastType::Warning => "⚠",
            ToastType::Info => "ℹ",
        }
    }

    pub fn icon_svg(&self) -> &'static str {
        match self {
            ToastType::Success => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                </svg>"#
            },
            ToastType::Error => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18.3 5.71c-.39-.39-1.02-.39-1.41 0L12 10.59 7.11 5.7c-.39-.39-1.02-.39-1.41 0-.39.39-.39 1.02 0 1.41L10.59 12 5.7 16.89c-.39.39-.39 1.02 0 1.41.39.39 1.02.39 1.41 0L12 13.41l4.89 4.88c.39.39 1.02.39 1.41 0 .39-.39.39-1.02 0-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z"/>
                </svg>"#
            },
            ToastType::Warning => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                </svg>"#
            },
            ToastType::Info => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                </svg>"#
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToastMessage {
    pub id: String,
    pub message: String,
    pub toast_type: ToastType,
    pub duration: u32, // in milliseconds
}

impl ToastMessage {
    pub fn success(message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            toast_type: ToastType::Success,
            duration: 2000, // 2 seconds
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            toast_type: ToastType::Error,
            duration: 3000, // 3 seconds for errors
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            toast_type: ToastType::Warning,
            duration: 2000, // 2 seconds
        }
    }

    pub fn info(message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            toast_type: ToastType::Info,
            duration: 2000, // 2 seconds
        }
    }
}

#[component]
pub fn Toast(
    toast: ToastMessage,
    #[prop(into)] on_close: Callback<String>,
) -> impl IntoView {
    let toast_ref = create_node_ref::<Div>();
    let (animation_state, set_animation_state) = create_signal("entering");

    // Show animation after mount
    create_effect(move |_| {
        if let Some(_element) = toast_ref.get() {
            // Small delay to allow DOM to render before animation
            set_timeout(
                move || {
                    set_animation_state.set("visible");
                },
                std::time::Duration::from_millis(50),
            );
        }
    });

    // Auto-remove after duration
    let toast_id = toast.id.clone();
    let on_close_clone = on_close.clone();
    create_effect(move |_| {
        let id = toast_id.clone();
        set_timeout(
            move || {
                set_animation_state.set("exiting");
                // Wait for exit animation before removing
                set_timeout(
                    move || {
                        on_close_clone.call(id.clone());
                    },
                    std::time::Duration::from_millis(400),
                );
            },
            std::time::Duration::from_millis(toast.duration as u64),
        );
    });

    let handle_close = move |_| {
        set_animation_state.set("exiting");
        let id = toast.id.clone();
        set_timeout(
            move || {
                on_close.call(id);
            },
            std::time::Duration::from_millis(400),
        );
    };

    // Clone toast_type values for use in multiple closures
    let toast_type_for_style = toast.toast_type;
    let toast_type_for_icon = toast.toast_type;
    let message = toast.message.clone();

    view! {
        <div
            node_ref=toast_ref
            class="toast-container"
            style=move || {
                let state = animation_state.get();
                let base_style = format!(
                    "
                    position: relative;
                    background: {};
                    border-left: 4px solid {};
                    color: {};
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
                    backdrop-filter: blur(8px);
                    min-width: 320px;
                    max-width: 480px;
                    margin-bottom: 12px;
                    overflow: hidden;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    ",
                    toast_type_for_style.background_gradient(),
                    toast_type_for_style.border_color(),
                    toast_type_for_style.text_color()
                );
                
                match state {
                    s if s == "entering" => format!("{} transform: translateX(100%) scale(0.9); opacity: 0; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style),
                    s if s == "visible" => format!("{} transform: translateX(0%) scale(1); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style),
                    s if s == "exiting" => format!("{} transform: translateX(100%) scale(0.95); opacity: 0; transition: all 0.4s cubic-bezier(0.5, 0, 0.75, 0);", base_style),
                    _ => base_style
                }
            }
        >
            // Progress bar
            <div 
                class="toast-progress"
                style=format!(
                    "
                    position: absolute;
                    top: 0;
                    left: 0;
                    height: 3px;
                    background: {};
                    animation: shrinkProgress {}ms linear;
                    transform-origin: left;
                    ",
                    toast_type_for_style.border_color(),
                    toast.duration
                )
            ></div>
            
            // Main content - Allineamento orizzontale migliorato
            <div style="
                display: flex; 
                align-items: center; 
                gap: 12px; 
                padding: 16px 20px;
                min-height: 52px;
            ">
                // Icon personalizzata con tick elegante
                <div style="
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    width: 24px;
                    height: 24px;
                    border-radius: 50%;
                    background: rgba(255, 255, 255, 0.2);
                    backdrop-filter: blur(4px);
                    flex-shrink: 0;
                " inner_html=toast_type_for_icon.icon_svg()>
                </div>
                
                // Message con allineamento perfetto
                <div style="
                    flex: 1;
                    font-size: 14px;
                    font-weight: 500;
                    line-height: 1.4;
                    display: flex;
                    align-items: center;
                    min-height: 20px;
                ">
                    {message}
                </div>
                
                // Close button allineato
                <button
                    on:click=handle_close
                    style="
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        width: 24px;
                        height: 24px;
                        background: rgba(255, 255, 255, 0.1);
                        border: none;
                        border-radius: 50%;
                        cursor: pointer;
                        opacity: 0.7;
                        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
                        flex-shrink: 0;
                        color: currentColor;
                    "
                    onmouseover="this.style.opacity='1'; this.style.transform='scale(1.1)'; this.style.background='rgba(255, 255, 255, 0.2)';"
                    onmouseout="this.style.opacity='0.7'; this.style.transform='scale(1)'; this.style.background='rgba(255, 255, 255, 0.1)';"
                    aria-label="Chiudi notifica"
                    title="Chiudi"
                >
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" style="display: block;">
                        <path d="M18.3 5.71c-.39-.39-1.02-.39-1.41 0L12 10.59 7.11 5.7c-.39-.39-1.02-.39-1.41 0-.39.39-.39 1.02 0 1.41L10.59 12 5.7 16.89c-.39.39-.39 1.02 0 1.41.39.39 1.02.39 1.41 0L12 13.41l4.89 4.88c.39.39 1.02.39 1.41 0 .39-.39.39-1.02 0-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z"/>
                    </svg>
                </button>
            </div>
        </div>
        
        // CSS Animation Styles
        <style>
            "
            @keyframes shrinkProgress {
                from {
                    transform: scaleX(1);
                }
                to {
                    transform: scaleX(0);
                }
            }
            
            @keyframes checkmark {
                0% {
                    transform: scale(0) rotate(0deg);
                    opacity: 0;
                }
                50% {
                    transform: scale(1.2) rotate(0deg);
                    opacity: 1;
                }
                100% {
                    transform: scale(1) rotate(0deg);
                    opacity: 1;
                }
            }
            
            @keyframes iconPulse {
                0%, 100% {
                    transform: scale(1);
                }
                50% {
                    transform: scale(1.05);
                }
            }
            
            .toast-container:hover .toast-progress {
                animation-play-state: paused;
            }
            
            .toast-container {
                will-change: transform, opacity;
            }
            
            /* Animazione per l'icona tick personalizzato */
            .toast-container svg {
                animation: checkmark 0.6s cubic-bezier(0.68, -0.55, 0.265, 1.55);
            }
            
            /* Pulse delicato per tutte le icone */
            .toast-container [style*='border-radius: 50%'] {
                animation: iconPulse 2s ease-in-out infinite;
            }
            
            /* Hover effect migliorato */
            .toast-container:hover {
                transform: translateX(-4px) !important;
                box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2) !important;
            }
            
            /* Allineamento perfetto per elementi */
            .toast-container * {
                box-sizing: border-box;
            }
            "
        </style>
    }
}


