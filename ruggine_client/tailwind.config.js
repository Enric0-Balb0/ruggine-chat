/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  darkMode: 'class', // Abilita dark mode tramite classe CSS
  theme: {
    extend: {
      colors: {
        // Brand colors - consistent in both themes
        'brand': {
          primary: '#464775',
          'primary-light': '#6264a7',
          'primary-dark': '#363567',
          secondary: '#323130',
          'secondary-light': '#605e5c',
        },
        // Background colors with dark variants
        'bg': {
          main: '#f3f2f1',
          sidebar: '#f8f8f8',
          card: '#ffffff',
          // Dark theme backgrounds
          'main-dark': '#1f1f1f',
          'sidebar-dark': '#2a2a2a',
          'card-dark': '#333333',
        },
        // Border colors for both themes
        'border': {
          DEFAULT: '#e1dfdd',
          light: '#c8c6c4',
          dark: '#404040',
          'dark-light': '#555555',
        },
        // Text colors for both themes
        'text': {
          primary: '#323130',
          secondary: '#605e5c',
          muted: '#8a8886',
          // Dark theme text
          'primary-dark': '#ffffff',
          'secondary-dark': '#e1dfdd',
          'muted-dark': '#c8c6c4',
        },
        // Status colors (same for both themes)
        'status': {
          success: '#107c10',
          danger: '#c4314b',
          warning: '#f7630c',
        },
        // Avatar colors palette (same for both themes)
        'avatar': {
          1: '#6264a7',  // Viola principale
          2: '#107c10',  // Verde
          3: '#d83b01',  // Rosso
          4: '#00bcf2',  // Azzurro
          5: '#8764b8',  // Viola chiaro
          6: '#ca5010',  // Arancione
          7: '#038387',  // Teal
          8: '#8e8cd8',  // Lavanda
          9: '#498205',  // Verde scuro
          10: '#c239b3', // Magenta
          11: '#ff8c00', // Arancione scuro
          12: '#0078d4', // Blu
        },
        // Group colors palette (same for both themes)
        'group': {
          1: '#6264a7',  // Viola principale
          2: '#2563eb',  // Blu
          3: '#16a34a',  // Verde
          4: '#dc2626',  // Rosso
          5: '#ea580c',  // Arancione
          6: '#7c3aed',  // Viola scuro
          7: '#0891b2',  // Cyan
          8: '#be185d',  // Pink
          9: '#65a30d',  // Lime
          10: '#7c2d12', // Marrone
          11: '#374151', // Grigio scuro
          12: '#1e40af', // Blu scuro
        }
      },
      fontFamily: {
        'sans': ['Segoe UI', 'Tahoma', 'Geneva', 'Verdana', 'sans-serif'],
      },
      fontSize: {
        'xs': '11px',
        'sm': '12px', 
        'base': '14px',
        'lg': '16px',
        'xl': '20px',
      },
      spacing: {
        'xs': '4px',
        'sm': '8px',
        'md': '12px', 
        'lg': '16px',
        'xl': '20px',
        '2xl': '24px',
      }
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}
