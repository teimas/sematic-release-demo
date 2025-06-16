use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn draw_loading_overlay(f: &mut Frame, area: Rect, animation_frame: usize, message: Option<&str>) {
    // Check if this is a Gemini analysis process
    let is_gemini_analysis = message.map_or(false, |msg| 
        msg.contains("Analizando cambios") || 
        msg.contains("Generando descripción") || 
        msg.contains("analizando seguridad") ||
        msg.contains("Gemini")
    );
    
    if is_gemini_analysis {
        draw_gemini_analysis_overlay(f, area, animation_frame, message);
    } else {
        draw_simple_loading_overlay(f, area, animation_frame, message);
    }
}

fn draw_gemini_analysis_overlay(f: &mut Frame, area: Rect, animation_frame: usize, message: Option<&str>) {
    // Spinner characters for animation
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_char = spinner_chars[animation_frame % spinner_chars.len()];
    
    // Create a larger, more detailed overlay for Gemini analysis
    let overlay_area = centered_rect(70, 30, area);
    f.render_widget(Clear, overlay_area);
    
    // Main block with title
    let main_block = Block::default()
        .title(format!(" {} 🧠 Análisis Inteligente con Gemini AI ", spinner_char))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    f.render_widget(main_block, overlay_area);

    // Content area inside the block
    let content_area = Rect {
        x: overlay_area.x + 2,
        y: overlay_area.y + 2,
        width: overlay_area.width.saturating_sub(4),
        height: overlay_area.height.saturating_sub(4),
    };

    // Determine current stage and create progress content
    let (current_stage, stage_details) = match message {
        Some(msg) if msg.contains("Analizando cambios") => {
            ("Etapa 1/3: Análisis de Cambios", vec![
                "🔍 Examinando archivos modificados",
                "📊 Evaluando impacto de los cambios",
                "🔧 Identificando patrones de código",
            ])
        },
        Some(msg) if msg.contains("analizando seguridad") => {
            ("Etapa 2/3: Análisis Múltiple", vec![
                "📝 Generando descripción técnica detallada",
                "🔒 Analizando posibles riesgos de seguridad",
                "⚠️  Detectando cambios que rompen compatibilidad",
            ])
        },
        Some(msg) if msg.contains("completado") => {
            ("Etapa 3/3: Finalizando", vec![
                "✅ Descripción generada exitosamente",
                "✅ Análisis de seguridad completado",
                "✅ Verificación de cambios finalizada",
            ])
        },
        _ => {
            ("Iniciando análisis...", vec![
                "🚀 Preparando análisis inteligente",
                "🌐 Conectando con Gemini AI",
                "📡 Enviando datos para procesamiento",
            ])
        }
    };

    // Current stage title
    let stage_title = Paragraph::new(current_stage)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    
    let title_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 1,
    };
    f.render_widget(stage_title, title_area);

    // Progress indicator
    let progress_percent = match message {
        Some(msg) if msg.contains("Analizando cambios") => 15,
        Some(msg) if msg.contains("analizando seguridad") => 65,
        Some(msg) if msg.contains("completado") => 100,
        _ => ((animation_frame * 2) % 30) as u16,
    };
    
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .percent(progress_percent)
        .label(format!("{}%", progress_percent));
    
    let gauge_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: 1,
    };
    f.render_widget(gauge, gauge_area);

    // Detailed process information
    let mut info_lines = vec![
        Line::from(""),
        Line::from("🔄 Procesos en ejecución:"),
        Line::from(""),
    ];
    
    for (i, detail) in stage_details.iter().enumerate() {
        let style = if i == (animation_frame / 8) % stage_details.len() {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        info_lines.push(Line::from(Span::styled(format!("  {}", detail), style)));
    }

    info_lines.extend(vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("💡 ¿Qué está sucediendo?", 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Gemini AI está analizando tus cambios de código para:"),
        Line::from("• Generar una descripción técnica detallada"),
        Line::from("• Identificar posibles vulnerabilidades de seguridad"),
        Line::from("• Detectar cambios que puedan romper compatibilidad"),
        Line::from(""),
        Line::from(Span::styled("⏱️  Este proceso puede tomar unos segundos...", 
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
    ]);

    let info_paragraph = Paragraph::new(info_lines)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    
    let info_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height.saturating_sub(4),
    };
    f.render_widget(info_paragraph, info_area);
}

fn draw_simple_loading_overlay(f: &mut Frame, area: Rect, animation_frame: usize, message: Option<&str>) {
    // Spinner characters for animation
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_char = spinner_chars[animation_frame % spinner_chars.len()];
    
    // Determine the loading message
    let loading_message = match message {
        Some(msg) if msg.contains("search") => "🔍 Buscando tareas en Monday.com...",
        Some(msg) if msg.contains("release") => "📝 Generando notas de versión...",
        _ => "⏳ Cargando...",
    };
    
    let loading_area = centered_rect(50, 15, area);
    f.render_widget(Clear, loading_area);
    
    // Main loading block
    let block = Block::default()
        .title(format!(" {} Procesando ", spinner_char))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, loading_area);

    // Create content area inside the block
    let content_area = Rect {
        x: loading_area.x + 2,
        y: loading_area.y + 2,
        width: loading_area.width.saturating_sub(4),
        height: loading_area.height.saturating_sub(4),
    };

    // Loading message
    let message_para = Paragraph::new(loading_message)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    let message_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 2,
    };
    f.render_widget(message_para, message_area);

    // Animated progress bar with pulsing effect
    let progress_percent = ((animation_frame * 3) % 100) as u16;
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .percent(progress_percent)
        .label(format!("{}%", progress_percent));
    
    let gauge_area = Rect {
        x: content_area.x,
        y: content_area.y + 3,
        width: content_area.width,
        height: 1,
    };
    f.render_widget(gauge, gauge_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 