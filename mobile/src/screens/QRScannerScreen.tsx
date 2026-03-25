import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Alert,
  Dimensions,
} from 'react-native';
import { CameraView, useCameraPermissions } from 'expo-camera';
import { useNavigation } from '@react-navigation/native';
import { apiClient } from '../api/client';

const { width } = Dimensions.get('window');
const SCANNER_SIZE = width * 0.7;

export function QRScannerScreen() {
  const navigation = useNavigation();
  const [permission, requestPermission] = useCameraPermissions();
  const [scanned, setScanned] = useState(false);
  const [isValidating, setIsValidating] = useState(false);

  useEffect(() => {
    if (!permission?.granted) {
      requestPermission();
    }
  }, [permission]);

  const handleBarCodeScanned = async ({ type, data }: { type: string; data: string }) => {
    if (scanned || isValidating) return;

    setScanned(true);
    setIsValidating(true);

    try {
      // Validate ticket via API
      const response = await apiClient.post('/tickets/validate', {
        qr_code: data,
        // TODO: Add vehicle_id and location when available
      });

      const result = response.data;

      if (result.valid) {
        Alert.alert(
          '✅ Bilet ważny',
          `Bilet został pomyślnie zweryfikowany.\n\nTyp: ${result.ticket?.ticket_type}\nCena: ${result.ticket?.price} ${result.ticket?.currency}`,
          [
            {
              text: 'Skanuj kolejny',
              onPress: () => setScanned(false),
            },
            {
              text: 'Zamknij',
              onPress: () => navigation.goBack(),
            },
          ]
        );
      } else {
        Alert.alert(
          '❌ Bilet nieważny',
          result.message || 'Ten bilet nie może być użyty.',
          [
            {
              text: 'Spróbuj ponownie',
              onPress: () => setScanned(false),
            },
            {
              text: 'Zamknij',
              onPress: () => navigation.goBack(),
            },
          ]
        );
      }
    } catch (error: any) {
      Alert.alert(
        'Błąd',
        error.response?.data?.error || 'Nie udało się zweryfikować biletu',
        [
          {
            text: 'Spróbuj ponownie',
            onPress: () => setScanned(false),
          },
        ]
      );
    } finally {
      setIsValidating(false);
    }
  };

  if (!permission?.granted) {
    return (
      <View style={styles.container}>
        <Text style={styles.message}>Potrzebny dostęp do kamery</Text>
        <TouchableOpacity style={styles.button} onPress={requestPermission}>
          <Text style={styles.buttonText}>Udziel dostępu</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <CameraView
        style={styles.camera}
        facing="back"
        barcodeScannerSettings={{
          barcodeTypes: ['qr'],
        }}
        onBarcodeScanned={scanned ? undefined : handleBarCodeScanned}
      >
        <View style={styles.overlay}>
          <View style={styles.scannerFrame}>
            <View style={[styles.corner, styles.topLeft]} />
            <View style={[styles.corner, styles.topRight]} />
            <View style={[styles.corner, styles.bottomLeft]} />
            <View style={[styles.corner, styles.bottomRight]} />
          </View>

          <Text style={styles.instructions}>
            {isValidating 
              ? 'Weryfikowanie biletu...' 
              : 'Zeskanuj kod QR biletu'
            }
          </Text>

          {scanned && (
            <TouchableOpacity
              style={styles.scanAgainButton}
              onPress={() => setScanned(false)}
            >
              <Text style={styles.scanAgainText}>Skanuj ponownie</Text>
            </TouchableOpacity>
          )}
        </View>
      </CameraView>

      <TouchableOpacity
        style={styles.closeButton}
        onPress={() => navigation.goBack()}
      >
        <Text style={styles.closeButtonText}>✕</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#000',
  },
  camera: {
    flex: 1,
  },
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  scannerFrame: {
    width: SCANNER_SIZE,
    height: SCANNER_SIZE,
    position: 'relative',
  },
  corner: {
    position: 'absolute',
    width: 40,
    height: 40,
    borderColor: '#2563EB',
    borderWidth: 4,
  },
  topLeft: {
    top: 0,
    left: 0,
    borderRightWidth: 0,
    borderBottomWidth: 0,
  },
  topRight: {
    top: 0,
    right: 0,
    borderLeftWidth: 0,
    borderBottomWidth: 0,
  },
  bottomLeft: {
    bottom: 0,
    left: 0,
    borderRightWidth: 0,
    borderTopWidth: 0,
  },
  bottomRight: {
    bottom: 0,
    right: 0,
    borderLeftWidth: 0,
    borderTopWidth: 0,
  },
  instructions: {
    color: '#fff',
    fontSize: 18,
    marginTop: 32,
    textAlign: 'center',
  },
  scanAgainButton: {
    marginTop: 24,
    backgroundColor: '#2563EB',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 8,
  },
  scanAgainText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  closeButton: {
    position: 'absolute',
    top: 50,
    right: 20,
    width: 44,
    height: 44,
    backgroundColor: 'rgba(0,0,0,0.5)',
    borderRadius: 22,
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    color: '#fff',
    fontSize: 24,
    fontWeight: 'bold',
  },
  message: {
    color: '#fff',
    fontSize: 18,
    textAlign: 'center',
    marginBottom: 20,
  },
  button: {
    backgroundColor: '#2563EB',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 8,
    alignSelf: 'center',
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
