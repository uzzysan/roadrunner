import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import axios from 'axios';
import { CameraView, useCameraPermissions } from 'expo-camera';
import React, { useState } from 'react';
import {
  ActivityIndicator,
  Linking,
  StyleSheet,
  View,
  useWindowDimensions,
} from 'react-native';
import { useTranslation } from 'react-i18next';
import { apiClient } from '../api/client';
import { AppText } from '../components/AppText';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { EmptyState } from '../components/EmptyState';
import { ErrorState } from '../components/ErrorState';
import { IconButton } from '../components/IconButton';
import { Screen } from '../components/Screen';
import { StatusBadge } from '../components/StatusBadge';
import { useTheme } from '../hooks/useTheme';
import type { RootStackParamList } from '../types';

type QRScannerNavigationProp = NativeStackNavigationProp<RootStackParamList, 'QRScanner'>;

interface ValidationTicket {
  id?: string;
  ticket_type?: string;
  price?: number;
  currency?: string;
  valid_until?: string;
}

interface ValidationPayload {
  valid?: boolean;
  ticket?: ValidationTicket | null;
}

export type ScannerResult =
  | { kind: 'valid'; ticket?: ValidationTicket | null }
  | { kind: 'invalid'; ticket?: ValidationTicket | null }
  | { kind: 'unverified'; reason: 'offline' | 'server' };

export function classifyValidationPayload(payload: ValidationPayload): ScannerResult {
  if (payload?.valid === true) return { kind: 'valid', ticket: payload.ticket };
  if (payload?.valid === false) return { kind: 'invalid', ticket: payload.ticket };
  return { kind: 'unverified', reason: 'server' };
}

export function classifyValidationFailure(error: unknown): ScannerResult {
  return {
    kind: 'unverified',
    reason: axios.isAxiosError(error) && !error.response ? 'offline' : 'server',
  };
}

export function QRScannerScreen() {
  const { t, i18n } = useTranslation();
  const navigation = useNavigation<QRScannerNavigationProp>();
  const [permission, requestPermission] = useCameraPermissions();
  const { colors, theme } = useTheme();
  const { width } = useWindowDimensions();
  const [scanned, setScanned] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [torchEnabled, setTorchEnabled] = useState(false);
  const [permissionError, setPermissionError] = useState(false);
  const [result, setResult] = useState<ScannerResult>();
  const frameSize = Math.max(240, Math.min(width - theme.space.xxl, 360));

  const close = () => {
    if (navigation.canGoBack()) navigation.goBack();
  };

  const askForPermission = async () => {
    setPermissionError(false);
    try {
      await requestPermission();
    } catch {
      setPermissionError(true);
    }
  };

  const handleBarCodeScanned = async ({ data }: { type: string; data: string }) => {
    if (scanned || isValidating) return;
    setScanned(true);
    setIsValidating(true);
    try {
      const response = await apiClient.post<ValidationPayload>('/tickets/validate', {
        qr_code: data,
      });
      setResult(classifyValidationPayload(response.data));
    } catch (error: unknown) {
      setResult(classifyValidationFailure(error));
    } finally {
      setIsValidating(false);
    }
  };

  const scanAgain = () => {
    setResult(undefined);
    setScanned(false);
  };

  const resultTitle = result?.kind === 'valid'
    ? t('scanner.validTitle')
    : result?.kind === 'invalid'
      ? t('scanner.invalidTitle')
      : t('scanner.unverifiedTitle');
  const resultDescription = result?.kind === 'valid'
    ? t('scanner.validDescription')
    : result?.kind === 'invalid'
      ? t('scanner.invalidDescription')
      : result?.reason === 'offline'
        ? t('scanner.unverifiedOfflineDescription')
        : t('scanner.unverifiedServerDescription');
  const resultTone = result?.kind === 'valid'
    ? 'success'
    : result?.kind === 'invalid'
      ? 'danger'
      : 'offline';
  const formatPrice = (ticket: ValidationTicket) => {
    if (typeof ticket.price !== 'number' || !ticket.currency) return undefined;
    try {
      return new Intl.NumberFormat(i18n.language, {
        style: 'currency',
        currency: ticket.currency,
      }).format(ticket.price);
    } catch {
      return `${ticket.price} ${ticket.currency}`;
    }
  };
  const formatValidity = (value?: string) => {
    if (!value) return undefined;
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return undefined;
    return new Intl.DateTimeFormat(i18n.language, {
      dateStyle: 'medium',
      timeStyle: 'short',
    }).format(date);
  };

  if (!permission) {
    return (
      <Screen contentContainerStyle={[styles.center, { gap: theme.space.md }]}>
        <ActivityIndicator accessibilityLabel={t('scanner.permissionChecking')} color={colors.primary} />
        <AppText tone="secondary">{t('scanner.permissionChecking')}</AppText>
      </Screen>
    );
  }

  if (!permission.granted) {
    return (
      <Screen
        contentContainerStyle={[styles.permission, { gap: theme.space.xl }]}
      >
        <EmptyState
          description={permission.canAskAgain
            ? t('scanner.permissionDescription')
            : t('scanner.permissionSettingsDescription')}
          icon="camera-outline"
          title={t('scanner.permissionTitle')}
        />
        {permissionError ? (
          <ErrorState
            description={t('scanner.permissionRequestError')}
            onRetry={() => void askForPermission()}
          />
        ) : null}
        <Button
          fullWidth
          icon={permission.canAskAgain ? 'camera-outline' : 'settings-outline'}
          label={permission.canAskAgain
            ? t('scanner.grantPermission')
            : t('scanner.openSettings')}
          onPress={permission.canAskAgain
            ? () => void askForPermission()
            : () => void Linking.openSettings()}
          safety
        />
        {navigation.canGoBack() ? (
          <Button
            fullWidth
            label={t('common.close')}
            onPress={close}
            safety
            variant="secondary"
          />
        ) : null}
      </Screen>
    );
  }

  if (result) {
    const ticket = result.kind === 'unverified' ? undefined : result.ticket;
    const price = ticket ? formatPrice(ticket) : undefined;
    const validity = formatValidity(ticket?.valid_until);
    return (
      <Screen
        scroll
        contentContainerStyle={[styles.result, { gap: theme.space.xl }]}
      >
        <Card
          accessibilityLiveRegion="assertive"
          elevated
          style={{ gap: theme.space.lg }}
        >
          <StatusBadge label={resultTitle} tone={resultTone} />
          <AppText accessibilityRole="header" variant="h1">{resultTitle}</AppText>
          <AppText tone="secondary">{resultDescription}</AppText>
          {ticket ? (
            <View style={{ gap: theme.space.xs }}>
              {ticket.ticket_type ? (
                <AppText>{t('scanner.ticketType', {
                  type: t(`tickets.type.${ticket.ticket_type}`, {
                    defaultValue: t('tickets.type.unknown'),
                  }),
                })}</AppText>
              ) : null}
              {ticket.id ? <AppText>{t('tickets.identifier', { id: ticket.id })}</AppText> : null}
              {validity ? (
                <AppText>{t('tickets.validUntil')}: {validity}</AppText>
              ) : null}
              {price ? <AppText variant="bodyStrong">{price}</AppText> : null}
            </View>
          ) : null}
          <Button
            fullWidth
            icon="scan-outline"
            label={t('scanner.scanAgain')}
            onPress={scanAgain}
            safety
          />
          {navigation.canGoBack() ? (
            <Button
              fullWidth
              label={t('common.close')}
              onPress={close}
              safety
              variant="secondary"
            />
          ) : null}
        </Card>
      </Screen>
    );
  }

  return (
    <View style={[styles.cameraScreen, { backgroundColor: colors.inverse }]}>
      <CameraView
        accessible={false}
        barcodeScannerSettings={{ barcodeTypes: ['qr'] }}
        enableTorch={torchEnabled}
        facing="back"
        onBarcodeScanned={scanned ? undefined : handleBarCodeScanned}
        style={styles.camera}
      >
        <View style={[styles.overlay, { backgroundColor: colors.overlay }]}>
          <View
            accessibilityLabel={t('scanner.frameLabel')}
            accessibilityRole="image"
            style={[
              styles.frame,
              {
                borderColor: colors.primary,
                borderRadius: theme.radius.lg,
                height: frameSize,
                width: frameSize,
              },
            ]}
          />
          <View style={[styles.instructions, { gap: theme.space.md }]}>
            {isValidating ? (
              <ActivityIndicator accessibilityLabel={t('scanner.validating')} color={colors.onInverse} />
            ) : null}
            <AppText style={styles.centerText} tone="inverse" variant="h3">
              {isValidating ? t('scanner.validating') : t('scanner.instructions')}
            </AppText>
          </View>
        </View>
      </CameraView>
      <View style={[styles.topActions, {
        gap: theme.space.sm,
        right: theme.space.lg,
        top: theme.space.lg,
      }]}>
        <IconButton
          accessibilityLabel={torchEnabled ? t('scanner.turnOffTorch') : t('scanner.turnOnTorch')}
          color={colors.text}
          icon={torchEnabled ? 'flash' : 'flash-outline'}
          onPress={() => setTorchEnabled((enabled) => !enabled)}
          safety
          selected={torchEnabled}
          style={{ backgroundColor: colors.surfaceRaised }}
        />
        <IconButton
          accessibilityLabel={t('common.close')}
          color={colors.text}
          icon="close"
          onPress={close}
          safety
          style={{ backgroundColor: colors.surfaceRaised }}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  camera: { flex: 1 },
  cameraScreen: { flex: 1 },
  center: { alignItems: 'center', justifyContent: 'center' },
  centerText: { textAlign: 'center' },
  frame: { borderWidth: 4 },
  instructions: { alignItems: 'center', maxWidth: '85%' },
  overlay: { alignItems: 'center', flex: 1, justifyContent: 'center' },
  permission: { justifyContent: 'center' },
  result: { justifyContent: 'center' },
  topActions: { flexDirection: 'row', position: 'absolute' },
});
