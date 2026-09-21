import { Ionicons } from '@expo/vector-icons';
import React, { useState, type ComponentProps } from 'react';
import { useTranslation } from 'react-i18next';
import { StyleSheet, TextInput, type TextInputProps, View } from 'react-native';
import { useTheme } from '../hooks/useTheme';
import { AppText } from './AppText';
import { IconButton } from './IconButton';

export interface TextFieldProps extends TextInputProps {
  label: string;
  helperText?: string;
  errorMessage?: string;
  required?: boolean;
  leadingIcon?: ComponentProps<typeof Ionicons>['name'];
  allowPasswordToggle?: boolean;
  showPasswordLabel?: string;
  hidePasswordLabel?: string;
}

export function TextField({
  allowPasswordToggle = false,
  editable = true,
  errorMessage,
  helperText,
  hidePasswordLabel = 'Hide password',
  label,
  leadingIcon,
  required = false,
  secureTextEntry,
  showPasswordLabel = 'Show password',
  style,
  ...props
}: TextFieldProps) {
  const { colors, theme } = useTheme();
  const { t } = useTranslation();
  const [passwordVisible, setPasswordVisible] = useState(false);
  const description = errorMessage ?? helperText;
  const isPassword = allowPasswordToggle && secureTextEntry;

  return (
    <View style={styles.container}>
      <AppText variant="label">
        {label}{required ? ` — ${t('validation.required')}` : ''}
      </AppText>
      <View style={[
        styles.inputFrame,
        {
          backgroundColor: colors.surface,
          borderColor: errorMessage ? colors.danger : colors.borderStrong,
          borderRadius: theme.radius.md,
          minHeight: theme.size.input,
        },
        !editable && { backgroundColor: colors.disabled },
      ]}>
        {leadingIcon ? (
          <Ionicons
            accessibilityElementsHidden
            color={colors.textSecondary}
            importantForAccessibility="no-hide-descendants"
            name={leadingIcon}
            size={theme.size.icon}
          />
        ) : null}
        <TextInput
          accessibilityLabel={label}
          accessibilityState={{ disabled: !editable }}
          editable={editable}
          placeholderTextColor={colors.textMuted}
          secureTextEntry={Boolean(secureTextEntry && !passwordVisible)}
          style={[styles.input, { color: editable ? colors.text : colors.onDisabled }, style]}
          {...props}
        />
        {isPassword ? (
          <IconButton
            accessibilityLabel={passwordVisible ? hidePasswordLabel : showPasswordLabel}
            icon={passwordVisible ? 'eye-off-outline' : 'eye-outline'}
            onPress={() => setPasswordVisible((visible) => !visible)}
          />
        ) : null}
      </View>
      {description ? (
        <View accessibilityLiveRegion={errorMessage ? 'polite' : 'none'} style={styles.description}>
          {errorMessage ? <Ionicons color={colors.danger} name="alert-circle-outline" size={16} /> : null}
          <AppText tone={errorMessage ? 'danger' : 'muted'} variant="caption">
            {description}
          </AppText>
        </View>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { gap: 4 },
  description: { alignItems: 'center', flexDirection: 'row', gap: 4 },
  input: { flex: 1, fontSize: 16, lineHeight: 24, minHeight: 48, paddingHorizontal: 0 },
  inputFrame: { alignItems: 'center', borderWidth: 1, flexDirection: 'row', gap: 8, paddingHorizontal: 16 },
});
