/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      colors: {
        // Brand colors based on UI mock
        'brand': {
          primary: '#464775',
          'primary-light': '#6264a7',
          secondary: '#323130',
          'secondary-light': '#605e5c',
        },
        // Background colors
        'bg': {
          main: '#f3f2f1',
          sidebar: '#f8f8f8',
        },
        // Border colors  
        'border': {
          DEFAULT: '#e1dfdd',
          light: '#c8c6c4',
        },
        // Status colors
        'status': {
          success: '#107c10',
          danger: '#c4314b',
          warning: '#f7630c',
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
