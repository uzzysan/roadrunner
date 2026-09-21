import React from 'react';
import { Image, type ImageStyle, type StyleProp } from 'react-native';
import { useTheme } from '../hooks/useTheme';

const assets = {
  light: {
    mark: require('../../assets/brand/mark-light.png'),
    lockup: require('../../assets/brand/lockup-light.png'),
  },
  dark: {
    mark: require('../../assets/brand/mark-dark.png'),
    lockup: require('../../assets/brand/lockup-dark.png'),
  },
};

export interface BrandMarkProps {
  variant?: 'mark' | 'lockup';
  width?: number;
  accessibilityLabel?: string;
  decorative?: boolean;
  style?: StyleProp<ImageStyle>;
}

export function BrandMark({
  accessibilityLabel = 'RoadRunner',
  decorative = false,
  style,
  variant = 'lockup',
  width = variant === 'mark' ? 32 : 176,
}: BrandMarkProps) {
  const { isDark } = useTheme();
  const ratio = variant === 'mark' ? 1 : 364 / 64;
  return (
    <Image
      accessibilityElementsHidden={decorative}
      accessibilityIgnoresInvertColors
      accessibilityLabel={decorative ? undefined : accessibilityLabel}
      accessible={!decorative}
      importantForAccessibility={decorative ? 'no-hide-descendants' : 'yes'}
      resizeMode="contain"
      source={assets[isDark ? 'dark' : 'light'][variant]}
      style={[{ height: width / ratio, width }, style]}
    />
  );
}
