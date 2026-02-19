// Kolory RoadRunner - zgodne z brandem

export const brandColors = {
  // Pomarańczowy - kolor logo i akcentów
  orange: '#F7941D',
  orangeDark: '#E8850F',
  orangeLight: '#FFB74D',
  
  // Szare
  grayDark: '#43526B',   // Ciemny szary - tekst w jasnym motywie
  grayLight: '#E7ECEF',  // Jasny szary - tło w jasnym motywie
  
  // Dodatkowe
  white: '#FFFFFF',
  black: '#1A1A1A',
};

// Gradienty dla dolnej części ekranu
export const gradients = {
  // Jasny: od delikatnie ciemniejszego niż tło
  light: ['#E7ECEF', '#DDE3E7', '#D3DADF'] as const,
  // Ciemny: od delikatnie jaśniejszego niż tło
  dark: ['#43526B', '#4A5A75', '#52627F'] as const,
};

// Light Theme
export const lightTheme = {
  dark: false,
  colors: {
    // Primary - pomarańczowy
    primary: brandColors.orange,
    onPrimary: '#FFFFFF',
    primaryContainer: '#FFF3E0',
    onPrimaryContainer: brandColors.orangeDark,
    
    // Secondary - ciemny szary
    secondary: brandColors.grayDark,
    onSecondary: '#FFFFFF',
    secondaryContainer: '#E3E8EC',
    onSecondaryContainer: brandColors.grayDark,
    
    // Tło - jasny szary
    background: brandColors.grayLight,
    onBackground: brandColors.grayDark,
    
    // Surface - biały (karty)
    surface: brandColors.white,
    onSurface: brandColors.grayDark,
    surfaceVariant: '#F5F7F8',
    onSurfaceVariant: '#5A6A7D',
    
    // Outline
    outline: '#C5CDD3',
    outlineVariant: '#DDE3E7',
    
    // Statusy
    error: '#D32F2F',
    onError: '#FFFFFF',
    success: '#2E7D32',
    warning: brandColors.orange,
    info: '#1976D2',
    
    // Header
    header: 'transparent',
    onHeader: brandColors.grayDark,
  },
};

// Dark Theme
export const darkTheme = {
  dark: true,
  colors: {
    // Primary - pomarańczowy (jaśniejszy)
    primary: '#FFB74D',
    onPrimary: '#1A1A1A',
    primaryContainer: brandColors.orangeDark,
    onPrimaryContainer: '#FFFFFF',
    
    // Secondary - jasny szary
    secondary: brandColors.grayLight,
    onSecondary: brandColors.grayDark,
    secondaryContainer: '#4A5A75',
    onSecondaryContainer: '#FFFFFF',
    
    // Tło - ciemny szary
    background: brandColors.grayDark,
    onBackground: brandColors.grayLight,
    
    // Surface - ciemniejszy (karty)
    surface: '#4A5A75',
    onSurface: brandColors.grayLight,
    surfaceVariant: '#52627F',
    onSurfaceVariant: '#B0BCC8',
    
    // Outline
    outline: '#5A6A7D',
    outlineVariant: '#52627F',
    
    // Statusy
    error: '#EF5350',
    onError: '#1A1A1A',
    success: '#66BB6A',
    warning: '#FFA726',
    info: '#42A5F5',
    
    // Header
    header: 'transparent',
    onHeader: brandColors.grayLight,
  },
};
