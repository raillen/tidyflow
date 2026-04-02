using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Media.Imaging;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using Net.Codecrete.QrCodeGenerator;

namespace FolderFlow.App.ViewModels;

public partial class DonateViewModel : ViewModelBase
{
    private readonly ILocalizationService _localizationService;
    
    [ObservableProperty] private Bitmap? _qrCode;
    [ObservableProperty] private string _pixKey = "contato@raillen.site";
    [ObservableProperty] private string _coffeeUrl = "https://www.buymeacoffee.com/raillen";

    public DonateViewModel(ILocalizationService localizationService)
    {
        _localizationService = localizationService;
        GenerateQrCode();
    }

    private void GenerateQrCode()
    {
        try
        {
            // Gera QR Code para a chave PIX
            var qr = QrCode.EncodeText(PixKey, QrCode.Ecc.Medium);
            var size = 300;
            var scale = size / qr.Size;
            
            // Criamos um bitmap simples (Preto e Branco)
            // Para simplificar no Avalonia sem depender de System.Drawing,
            // vamos gerar um array de pixels ou usar um WriteableBitmap se fosse complexo.
            // Aqui vamos usar a estratégia de gerar um SVG e carregar se possível, 
            // ou melhor, gerar um WriteableBitmap manualmente.
            
            var pixelSize = qr.Size;
            var bitmap = new WriteableBitmap(new PixelSize(pixelSize, pixelSize), new Vector(96, 96), Avalonia.Platform.PixelFormat.Rgba8888, Avalonia.Platform.AlphaFormat.Opaque);
            
            using (var lockedBitmap = bitmap.Lock())
            {
                unsafe
                {
                    var ptr = (uint*)lockedBitmap.Address;
                    for (int y = 0; y < pixelSize; y++)
                    {
                        for (int x = 0; x < pixelSize; x++)
                        {
                            bool black = qr.GetModule(x, y);
                            ptr[y * pixelSize + x] = black ? 0xFF000000 : 0xFFFFFFFF;
                        }
                    }
                }
            }
            
            QrCode = bitmap;
        }
        catch { }
    }

    [RelayCommand]
    private async Task CopyPixKey()
    {
        if (Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            var clipboard = desktop.MainWindow?.Clipboard;
            if (clipboard != null)
            {
                await clipboard.SetTextAsync(PixKey);
            }
        }
    }

    [RelayCommand]
    private void OpenCoffee()
    {
        try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(CoffeeUrl) { UseShellExecute = true }); } catch { }
    }
}
