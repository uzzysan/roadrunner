import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useTranslation } from 'react-i18next';
import { useNavigation } from '@react-navigation/native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { MainTabParamList } from '../navigation/AppNavigator';
import { useAuthStore } from '../store/authStore';
import { apiClient } from '../api/client';

type BuyTicketScreenNavigationProp = NativeStackNavigationProp<MainTabParamList, 'Tickets'>;

interface TicketType {
  id: string;
  type: 'single' | 'weekly' | 'monthly' | 'discounted';
  name: string;
  price: number;
  validityDays: number;
  description: string;
  icon: string;
}

const TICKET_TYPES: TicketType[] = [
  {
    id: 'single',
    type: 'single',
    name: 'Bilet jednorazowy',
    price: 5.00,
    validityDays: 1,
    description: 'Ważny przez 24h od zakupu',
    icon: '🎫',
  },
  {
    id: 'weekly',
    type: 'weekly',
    name: 'Bilet tygodniowy',
    price: 25.00,
    validityDays: 7,
    description: 'Ważny przez 7 dni od zakupu',
    icon: '📅',
  },
  {
    id: 'monthly',
    type: 'monthly',
    name: 'Bilet miesięczny',
    price: 80.00,
    validityDays: 30,
    description: 'Ważny przez 30 dni od zakupu',
    icon: '📆',
  },
  {
    id: 'discounted',
    type: 'discounted',
    name: 'Bilet ulgowy',
    price: 2.50,
    validityDays: 1,
    description: 'Dla uczniów, studentów, seniorów',
    icon: '💳',
  },
];

export function BuyTicketScreen() {
  const { t } = useTranslation();
  const navigation = useNavigation<BuyTicketScreenNavigationProp>();
  const { accessToken } = useAuthStore();
  const [selectedTicket, setSelectedTicket] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleBuyTicket = async (ticketType: TicketType) => {
    if (!accessToken) {
      Alert.alert('Błąd', 'Musisz być zalogowany');
      return;
    }

    Alert.alert(
      'Potwierdź zakup',
      `Czy chcesz kupić ${ticketType.name} za ${ticketType.price.toFixed(2)} PLN?`,
      [
        { text: 'Anuluj', style: 'cancel' },
        {
          text: 'Kupuję',
          onPress: () => purchaseTicket(ticketType),
        },
      ]
    );
  };

  const purchaseTicket = async (ticketType: TicketType) => {
    try {
      setIsLoading(true);

      const response = await apiClient.post('/tickets', {
        ticket_type: ticketType.type,
      });

      const ticket = response.data;

      Alert.alert(
        'Sukces!',
        `Bilet został zakupiony pomyślnie.\n\nKod: ${ticket.id}`,
        [
          {
            text: 'Zobacz bilet',
            onPress: () => {
              // Navigate to ticket details
              navigation.navigate('Tickets');
            },
          },
          { text: 'OK', style: 'cancel' },
        ]
      );
    } catch (error: any) {
      Alert.alert(
        'Błąd',
        error.response?.data?.error || 'Nie udało się kupić biletu'
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Kup bilet</Text>
        <Text style={styles.subtitle}>Wybierz rodzaj biletu</Text>
      </View>

      <View style={styles.ticketsContainer}>
        {TICKET_TYPES.map((ticket) => (
          <TouchableOpacity
            key={ticket.id}
            style={[
              styles.ticketCard,
              selectedTicket === ticket.id && styles.ticketCardSelected,
            ]}
            onPress={() => handleBuyTicket(ticket)}
            disabled={isLoading}
          >
            <View style={styles.ticketHeader}>
              <Text style={styles.ticketIcon}>{ticket.icon}</Text>
              <View style={styles.ticketInfo}>
                <Text style={styles.ticketName}>{ticket.name}</Text>
                <Text style={styles.ticketDescription}>
                  {ticket.description}
                </Text>
              </View>
            </View>

            <View style={styles.ticketFooter}>
              <Text style={styles.ticketPrice}>
                {ticket.price.toFixed(2)} PLN
              </Text>
              <TouchableOpacity
                style={styles.buyButton}
                onPress={() => handleBuyTicket(ticket)}
                disabled={isLoading}
              >
                <Text style={styles.buyButtonText}>
                  {isLoading ? '...' : 'Kup'}
                </Text>
              </TouchableOpacity>
            </View>
          </TouchableOpacity>
        ))}
      </View>

      <View style={styles.infoSection}>
        <Text style={styles.infoTitle}>Informacje</Text>
        <Text style={styles.infoText}>
          • Bilet jest ważny od momentu zakupu{'
'}
          • Możesz użyć biletu w dowolnym pojeździe{'
'}
          • Pokaż kod QR kierowcy przy wejściu{'
'}
          • Bilet ulgowy wymaga okazania legitymacji
        </Text>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
  },
  header: {
    padding: 20,
    backgroundColor: '#2563EB',
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
  },
  subtitle: {
    fontSize: 16,
    color: '#DBEAFE',
    marginTop: 4,
  },
  ticketsContainer: {
    padding: 16,
  },
  ticketCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  ticketCardSelected: {
    borderColor: '#2563EB',
  },
  ticketHeader: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  ticketIcon: {
    fontSize: 32,
    marginRight: 12,
  },
  ticketInfo: {
    flex: 1,
  },
  ticketName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#0F172A',
  },
  ticketDescription: {
    fontSize: 14,
    color: '#64748B',
    marginTop: 4,
  },
  ticketFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#E2E8F0',
  },
  ticketPrice: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#2563EB',
  },
  buyButton: {
    backgroundColor: '#2563EB',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  buyButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  infoSection: {
    padding: 16,
    marginTop: 8,
  },
  infoTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#0F172A',
    marginBottom: 12,
  },
  infoText: {
    fontSize: 14,
    color: '#64748B',
    lineHeight: 22,
  },
});
