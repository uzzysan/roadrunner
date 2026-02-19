// Kolory wyciągnięte z logo RoadRunner

export const brandColors = {
  // Czerwony z autobusu (logo)
  busRed: '#E53935',
  busRedDark: '#C62828',
  busRedLight: '#FFEBEE',
  
  // Granatowy/czarny z opon i okien
  navy: '#1A1A2E',
  navyLight: '#2D2D44',
  
  // Szary z drogi
  roadGray: '#9E9E9E',
  roadGrayLight: '#E0E0E0',
  
  // Biały/szary z chmur
  cloud: '#F5F5F5',
  cloudDark: '#ECEFF1',
};

// Gradienty dla dolnej części ekranu
export const gradients = {
  light: ['#FFFFFF', '#FFEBEE', '#FFCDD2'] as const,  // Biały -> jasny czerwony
  dark: ['#1A1A2E', '#2D2D44', '#3D3D5C'] as const,    // Granat -> ciemny granat
};

// Light Theme
export const lightTheme = {
  dark: false,
  colors: {
    // Primary - czerwony autobusu
    primary: brandColors.busRed,
    onPrimary: '#FFFFFF',
    primaryContainer: brandColors.busRedLight,
    onPrimaryContainer: brandColors.busRedDark,
    
    // Secondary - granatowy
    secondary: brandColors.navy,
    onSecondary: '#FFFFFF',
    secondaryContainer: brandColors.navyLight,
    onSecondaryContainer: '#FFFFFF',
    
    // Tło
    background: '#FAFAFA',
    onBackground: '#212121',
    
    // Surface - karty
    surface: '#FFFFFF',
    onSurface: '#212121',
    surfaceVariant: '#F5F5F5',
    onSurfaceVariant: '#616161',
    
    // Outline
    outline: '#E0E0E0',
    outlineVariant: '#EEEEEE',
    
    // Statusy
    error: '#D32F2F',
    onError: '#FFFFFF',
    success: '#2E7D32',
    warning: '#ED6C02',
    info: '#0288D1',
    
    // Header - przezroczysty/jasny
    header: 'transparent',
    onHeader: brandColors.navy,
  },
};

// Dark Theme
export const darkTheme = {
  dark: true,
  colors: {
    // Primary - jaśniejszy czerwony
    primary: '#FF6659',
    onPrimary: '#000000',
    primaryContainer: brandColors.busRedDark,
    onPrimaryContainer: '#FFFFFF',
    
    // Secondary - jaśniejszy granat
    secondary: '#5C5C8A',
    onSecondary: '#FFFFFF',
    secondaryContainer: brandColors.navy,
    onSecondaryContainer: '#FFFFFF',
    
    // Tło
    background: brandColors.navy,
    onBackground: '#FFFFFF',
    
    // Surface - karty
    surface: brandColors.navyLight,
    onSurface: '#FFFFFF',
    surfaceVariant: '#3D3D5C',
    onSurfaceVariant: '#B0B0C3',
    
    // Outline
    outline: '#4A4A6A',
    outlineVariant: '#3D3D5C',
    
    // Statusy
    error: '#EF5350',
    onError: '#000000',
    success: '#66BB6A',
    warning: '#FFA726',
    info: '#42A5F5',
    
    // Header
    header: 'transparent',
    onHeader: '#FFFFFF',
  },
};
