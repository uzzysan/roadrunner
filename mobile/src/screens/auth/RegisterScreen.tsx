import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import axios from 'axios';
import React, { useState } from 'react';
import { KeyboardAvoidingView, Platform, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { authApi } from '../../api/client';
import { AppText } from '../../components/AppText';
import { BrandMark } from '../../components/BrandMark';
import { Button } from '../../components/Button';
import { Card } from '../../components/Card';
import { ErrorState } from '../../components/ErrorState';
import { Screen } from '../../components/Screen';
import { TextField } from '../../components/TextField';
import { useTheme } from '../../hooks/useTheme';
import { useAuthStore } from '../../store/authStore';
import type { RootStackParamList } from '../../types';

type RegisterScreenNavigationProp = NativeStackNavigationProp<RootStackParamList, 'Register'>;
type FieldName = 'firstName' | 'lastName' | 'email' | 'phone' | 'password' | 'confirmPassword';
type FormData = Record<FieldName, string>;
type FormErrors = Partial<Record<FieldName, string>>;
type RequestError = 'offline' | 'registration' | null;

interface ApiUser {
  id: string;
  email: string;
  first_name?: string;
  firstName?: string;
  last_name?: string;
  lastName?: string;
  role: string;
}

interface RegisterPayload {
  user?: ApiUser;
  access_token?: string;
  refresh_token?: string;
}

const initialForm: FormData = {
  email: '',
  password: '',
  confirmPassword: '',
  firstName: '',
  lastName: '',
  phone: '',
};

const isEmail = (value: string) => /^\S+@\S+\.\S+$/.test(value);
const isPhone = (value: string) => /^[+\d][\d\s()-]{5,}$/.test(value);

export function RegisterScreen() {
  const { t } = useTranslation();
  const navigation = useNavigation<RegisterScreenNavigationProp>();
  const { login, setLoading, isLoading } = useAuthStore();
  const { theme } = useTheme();
  const [formData, setFormData] = useState<FormData>(initialForm);
  const [errors, setErrors] = useState<FormErrors>({});
  const [requestError, setRequestError] = useState<RequestError>(null);

  const updateField = (field: FieldName, value: string) => {
    setFormData((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({ ...current, [field]: undefined }));
    setRequestError(null);
  };

  const validate = (): FormErrors => {
    const nextErrors: FormErrors = {};
    if (!formData.firstName.trim()) nextErrors.firstName = t('validation.firstNameRequired');
    if (!formData.lastName.trim()) nextErrors.lastName = t('validation.lastNameRequired');
    if (!formData.email.trim()) {
      nextErrors.email = t('validation.emailRequired');
    } else if (!isEmail(formData.email.trim())) {
      nextErrors.email = t('validation.emailInvalid');
    }
    if (!formData.password) {
      nextErrors.password = t('validation.passwordRequired');
    } else if (formData.password.length < 8) {
      nextErrors.password = t('validation.passwordMinLength');
    }
    if (!formData.confirmPassword) {
      nextErrors.confirmPassword = t('validation.confirmPasswordRequired');
    } else if (formData.password !== formData.confirmPassword) {
      nextErrors.confirmPassword = t('validation.passwordMatch');
    }
    if (formData.phone.trim() && !isPhone(formData.phone.trim())) {
      nextErrors.phone = t('validation.phoneInvalid');
    }
    return nextErrors;
  };

  const handleRegister = async () => {
    const nextErrors = validate();
    setErrors(nextErrors);
    setRequestError(null);
    if (Object.keys(nextErrors).length > 0) return;

    try {
      setLoading(true);
      const response = await authApi.register({
        email: formData.email.trim(),
        password: formData.password,
        first_name: formData.firstName.trim(),
        last_name: formData.lastName.trim(),
        phone: formData.phone.trim() || undefined,
      });
      const payload = response.data as RegisterPayload;
      if (!payload.user || !payload.access_token || !payload.refresh_token) {
        setRequestError('registration');
        return;
      }
      login({
        id: payload.user.id,
        email: payload.user.email,
        firstName: payload.user.firstName ?? payload.user.first_name ?? '',
        lastName: payload.user.lastName ?? payload.user.last_name ?? '',
        role: payload.user.role,
      }, payload.access_token, payload.refresh_token);
    } catch (error: unknown) {
      setRequestError(axios.isAxiosError(error) && !error.response ? 'offline' : 'registration');
    } finally {
      setLoading(false);
    }
  };

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      style={styles.fill}
    >
      <Screen
        scroll
        contentContainerStyle={[
          styles.content,
          { gap: theme.space.xl, paddingVertical: theme.space.xxl },
        ]}
        scrollViewProps={{ keyboardDismissMode: 'on-drag' }}
      >
        <BrandMark accessibilityLabel={t('brand.logoLabel')} />
        <Card style={[styles.form, { gap: theme.space.lg, maxWidth: theme.size.formMax }]}>
          <View style={{ gap: theme.space.sm }}>
            <AppText accessibilityRole="header" variant="h1">{t('auth.register')}</AppText>
            <AppText tone="secondary">{t('auth.registerDescription')}</AppText>
          </View>

          {requestError ? (
            <ErrorState
              description={requestError === 'offline'
                ? t('common.offlineDescription')
                : t('auth.registrationError')}
              onRetry={handleRegister}
              title={requestError === 'offline' ? t('common.offline') : t('common.error')}
            />
          ) : null}

          <TextField
            autoCapitalize="words"
            autoComplete="given-name"
            errorMessage={errors.firstName}
            label={t('auth.firstName')}
            onChangeText={(value) => updateField('firstName', value)}
            required
            textContentType="givenName"
            value={formData.firstName}
          />
          <TextField
            autoCapitalize="words"
            autoComplete="family-name"
            errorMessage={errors.lastName}
            label={t('auth.lastName')}
            onChangeText={(value) => updateField('lastName', value)}
            required
            textContentType="familyName"
            value={formData.lastName}
          />
          <TextField
            autoCapitalize="none"
            autoComplete="email"
            errorMessage={errors.email}
            keyboardType="email-address"
            label={t('auth.email')}
            leadingIcon="mail-outline"
            onChangeText={(value) => updateField('email', value)}
            required
            textContentType="emailAddress"
            value={formData.email}
          />
          <TextField
            autoComplete="tel"
            errorMessage={errors.phone}
            helperText={t('auth.phoneOptional')}
            keyboardType="phone-pad"
            label={t('auth.phone')}
            leadingIcon="call-outline"
            onChangeText={(value) => updateField('phone', value)}
            textContentType="telephoneNumber"
            value={formData.phone}
          />
          <TextField
            allowPasswordToggle
            autoComplete="new-password"
            errorMessage={errors.password}
            helperText={t('validation.passwordMinLength')}
            hidePasswordLabel={t('auth.hidePassword')}
            label={t('auth.password')}
            leadingIcon="lock-closed-outline"
            onChangeText={(value) => updateField('password', value)}
            required
            secureTextEntry
            showPasswordLabel={t('auth.showPassword')}
            textContentType="newPassword"
            value={formData.password}
          />
          <TextField
            allowPasswordToggle
            autoComplete="new-password"
            errorMessage={errors.confirmPassword}
            hidePasswordLabel={t('auth.hidePassword')}
            label={t('auth.confirmPassword')}
            leadingIcon="lock-closed-outline"
            onChangeText={(value) => updateField('confirmPassword', value)}
            onSubmitEditing={handleRegister}
            required
            returnKeyType="done"
            secureTextEntry
            showPasswordLabel={t('auth.showPassword')}
            textContentType="newPassword"
            value={formData.confirmPassword}
          />
          <Button
            fullWidth
            label={t('auth.registerButton')}
            loading={isLoading}
            loadingLabel={t('auth.registering')}
            onPress={handleRegister}
          />
          <Button
            disabled={isLoading}
            fullWidth
            label={t('auth.backToLogin')}
            onPress={() => navigation.navigate('Login')}
            variant="quiet"
          />
        </Card>
      </Screen>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  content: { alignItems: 'center' },
  fill: { flex: 1 },
  form: { alignSelf: 'center', width: '100%' },
});
