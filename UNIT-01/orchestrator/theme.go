package main

import "github.com/charmbracelet/lipgloss"

// Ruthen Labs Directive System Theme
// Primary Palette: Black, Grey, and Amber

var (
	// Base Colors
	ColorAmber     = lipgloss.Color("#FFB000") // Primary UI borders, prompts, buttons
	ColorLightGrey = lipgloss.Color("#A0A0A0") // Primary text, active states
	ColorDarkGrey  = lipgloss.Color("#444444") // Metadata, inactive states, paths
	ColorBlack     = lipgloss.Color("#000000") // Backgrounds (usually terminal default)
	
	// Specific Semantic Colors
	ColorWarning   = lipgloss.Color("#FF5555") // Errors, Rejections (Minimal usage)
	ColorSuccess   = lipgloss.Color("#FFB000") // We use Amber for success to keep the strict 3-color palette

	// Common Styles
	ThemeBrandStyle  = lipgloss.NewStyle().Foreground(ColorAmber).Bold(true)
	ThemeDimStyle    = lipgloss.NewStyle().Foreground(ColorDarkGrey)
	ThemeTextStyle   = lipgloss.NewStyle().Foreground(ColorLightGrey)
	ThemeBorderStyle = lipgloss.NewStyle().BorderStyle(lipgloss.RoundedBorder()).BorderForeground(ColorAmber)
	
	// Interactive / Form Styles
	ThemePromptStyle = lipgloss.NewStyle().Foreground(ColorAmber).Bold(true)
	ThemeToolStyle   = lipgloss.NewStyle().Foreground(ColorDarkGrey).Italic(true)
)
