// RoadRunner Brand Colors - updated, less aggressive
export const brandColors = {
  // Primary - czerwony autobus (mniej agresywny)
  primary: '#D32F2F',        // Ciemniejszy czerwony
  primaryDark: '#B71C1C',    // Dla dark mode
  primaryLight: '#EF5350',   // Dla light mode
  
  // Secondary - granat
  secondary: '#1A237E',
  secondaryDark: '#0D1642',
  secondaryLight: '#3949AB',
  
  // Accent - żółty/szkolny
  accent: '#FFC107',
  accentDark: '#FFA000',
  
  // Header - szary zamiast czerwonego
  header: '#37474F',         // Ciemny szary
  headerDark: '#263238',     // Dla dark mode
};

// Light Theme
export const lightTheme = {
  dark: false,
  colors: {
    primary: brandColors.primary,
    onPrimary: '#FFFFFF',
    primaryContainer: '#FFEBEE',  // Jasny czerwony
    onPrimaryContainer: brandColors.primaryDark,
    
    secondary: brandColors.secondary,
    onSecondary: '#FFFFFF',
    secondaryContainer: brandColors.secondaryLight,
    onSecondaryContainer: '#FFFFFF',
    
    tertiary: brandColors.accent,
    onTertiary: '#000000',
    
    background: '#FAFAFA',
    onBackground: '#212121',
    surface: '#FFFFFF',
    onSurface: '#212121',
    surfaceVariant: '#F5F5F5',
    onSurfaceVariant: '#616161',
    
    outline: '#BDBDBD',
    outlineVariant: '#E0E0E0',
    
    error: '#D32F2F',
    onError: '#FFFFFF',
    
    success: '#388E3C',
    warning: '#F57C00',
    info: '#1976D2',
    
    // Custom
    header: brandColors.header,
    onHeader: '#FFFFFF',
  },
};

// Dark Theme
export const darkTheme = {
  dark: true,
  colors: {
    primary: brandColors.primaryLight,
    onPrimary: '#FFFFFF',
    primaryContainer: brandColors.primaryDark,
    onPrimaryContainer: '#FFFFFF',
    
    secondary: brandColors.secondaryLight,
    onSecondary: '#FFFFFF',
    secondaryContainer: brandColors.secondaryDark,
    onSecondaryContainer: '#FFFFFF',
    
    tertiary: brandColors.accent,
    onTertiary: '#000000',
    
    background: '#121212',
    onBackground: '#FFFFFF',
    surface: '#1E1E1E',
    onSurface: '#FFFFFF',
    surfaceVariant: '#2C2C2C',
    onSurfaceVariant: '#BDBDBD',
    
    outline: '#616161',
    outlineVariant: '#424242',
    
    error: '#EF5350',
    onError: '#FFFFFF',
    
    success: '#66BB6A',
    warning: '#FFA726',
    info: '#42A5F5',
    
    // Custom
    header: brandColors.headerDark,
    onHeader: '#FFFFFF',
  },
};
