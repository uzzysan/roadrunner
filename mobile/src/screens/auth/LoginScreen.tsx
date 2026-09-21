import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useNavigation } from '@react-navigation/native';
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

type LoginScreenNavigationProp = NativeStackNavigationProp<RootStackParamList, 'Login'>;
type RequestError = 'offline' | 'credentials' | null;

interface ApiUser {
  id: string;
  email: string;
  first_name?: string;
  firstName?: string;
  last_name?: string;
  lastName?: string;
  role: string;
}

interface LoginPayload {
  user?: ApiUser;
  access_token?: string;
  refresh_token?: string;
  mfa_required?: boolean;
  temp_token?: string;
}

const isEmail = (value: string) => /^\S+@\S+\.\S+$/.test(value);

export function LoginScreen() {
  const { t } = useTranslation();
  const navigation = useNavigation<LoginScreenNavigationProp>();
  const { login, setLoading, isLoading } = useAuthStore();
  const { theme } = useTheme();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [mfaRequired, setMfaRequired] = useState(false);
  const [mfaCode, setMfaCode] = useState('');
  const [tempToken, setTempToken] = useState('');
  const [emailError, setEmailError] = useState<string>();
  const [passwordError, setPasswordError] = useState<string>();
  const [mfaError, setMfaError] = useState<string>();
  const [requestError, setRequestError] = useState<RequestError>(null);

  const completeLogin = (payload: LoginPayload) => {
    if (!payload.user || !payload.access_token || !payload.refresh_token) {
      setRequestError('credentials');
      return;
    }

    login({
      id: payload.user.id,
      email: payload.user.email,
      firstName: payload.user.firstName ?? payload.user.first_name ?? '',
      lastName: payload.user.lastName ?? payload.user.last_name ?? '',
      role: payload.user.role,
    }, payload.access_token, payload.refresh_token);
  };

  const handleLogin = async () => {
    const normalizedEmail = email.trim();
    const nextEmailError = !normalizedEmail
      ? t('validation.emailRequired')
      : !isEmail(normalizedEmail)
        ? t('validation.emailInvalid')
        : undefined;
    const nextPasswordError = password ? undefined : t('validation.passwordRequired');
    setEmailError(nextEmailError);
    setPasswordError(nextPasswordError);
    setRequestError(null);
    if (nextEmailError || nextPasswordError) return;

    try {
      setLoading(true);
      const response = await authApi.login(normalizedEmail, password);
      const payload = response.data as LoginPayload;
      if (payload.mfa_required) {
        if (!payload.temp_token) {
          setRequestError('credentials');
          return;
        }
        setMfaRequired(true);
        setTempToken(payload.temp_token);
        setPassword('');
      } else {
        completeLogin(payload);
      }
    } catch (error: unknown) {
      setRequestError(axios.isAxiosError(error) && !error.response ? 'offline' : 'credentials');
    } finally {
      setLoading(false);
    }
  };

  const handleMfaVerify = async () => {
    const nextMfaError = /^\d{6}$/.test(mfaCode)
      ? undefined
      : t('validation.mfaCodeInvalid');
    setMfaError(nextMfaError);
    setRequestError(null);
    if (nextMfaError) return;

    try {
      setLoading(true);
      const response = await authApi.verifyMfaLogin(tempToken, mfaCode);
      completeLogin(response.data as LoginPayload);
    } catch (error: unknown) {
      setRequestError(axios.isAxiosError(error) && !error.response ? 'offline' : 'credentials');
    } finally {
      setLoading(false);
    }
  };

  const resetMfa = () => {
    setMfaRequired(false);
    setMfaCode('');
    setMfaError(undefined);
    setTempToken('');
    setRequestError(null);
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
            <AppText accessibilityRole="header" variant="h1">
              {mfaRequired ? t('auth.mfaTitle') : t('auth.login')}
            </AppText>
            <AppText tone="secondary">
              {mfaRequired ? t('auth.mfaDescription') : t('auth.loginDescription')}
            </AppText>
          </View>

          {requestError ? (
            <ErrorState
              description={requestError === 'offline'
                ? t('common.offlineDescription')
                : t('auth.loginError')}
              onRetry={mfaRequired ? handleMfaVerify : handleLogin}
              title={requestError === 'offline' ? t('common.offline') : t('common.error')}
            />
          ) : null}

          {mfaRequired ? (
            <>
              <TextField
                autoComplete="one-time-code"
                autoFocus
                errorMessage={mfaError}
                keyboardType="number-pad"
                label={t('auth.mfaCode')}
                maxLength={6}
                onChangeText={(value) => {
                  setMfaCode(value.replace(/\D/g, ''));
                  setMfaError(undefined);
                }}
                required
                textContentType="oneTimeCode"
                value={mfaCode}
              />
              <Button
                fullWidth
                label={t('auth.verify')}
                loading={isLoading}
                loadingLabel={t('auth.verifying')}
                onPress={handleMfaVerify}
              />
              <Button
                disabled={isLoading}
                fullWidth
                label={t('auth.useDifferentAccount')}
                onPress={resetMfa}
                variant="quiet"
              />
            </>
          ) : (
            <>
              <TextField
                autoCapitalize="none"
                autoComplete="email"
                errorMessage={emailError}
                keyboardType="email-address"
                label={t('auth.email')}
                leadingIcon="mail-outline"
                onChangeText={(value) => {
                  setEmail(value);
                  setEmailError(undefined);
                }}
                required
                returnKeyType="next"
                textContentType="username"
                value={email}
              />
              <TextField
                allowPasswordToggle
                autoComplete="current-password"
                errorMessage={passwordError}
                hidePasswordLabel={t('auth.hidePassword')}
                label={t('auth.password')}
                leadingIcon="lock-closed-outline"
                onChangeText={(value) => {
                  setPassword(value);
                  setPasswordError(undefined);
                }}
                onSubmitEditing={handleLogin}
                required
                returnKeyType="done"
                secureTextEntry
                showPasswordLabel={t('auth.showPassword')}
                textContentType="password"
                value={password}
              />
              <Button
                fullWidth
                label={t('auth.loginButton')}
                loading={isLoading}
                loadingLabel={t('auth.loggingIn')}
                onPress={handleLogin}
              />
              <Button
                disabled={isLoading}
                fullWidth
                label={t('auth.createAccount')}
                onPress={() => navigation.navigate('Register')}
                variant="quiet"
              />
            </>
          )}
        </Card>
      </Screen>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  content: { alignItems: 'center', justifyContent: 'center' },
  fill: { flex: 1 },
  form: { alignSelf: 'center', width: '100%' },
});
