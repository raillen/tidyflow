using System;
using Avalonia;
using Avalonia.Data;
using Avalonia.Markup.Xaml;
using Avalonia.Markup.Xaml.MarkupExtensions;
using FolderFlow.Application.Interfaces;
using Microsoft.Extensions.DependencyInjection;

namespace FolderFlow.App.Localization;

public class LclExtension : MarkupExtension
{
    public string Key { get; set; }

    public LclExtension(string key)
    {
        Key = key;
    }

    public override object ProvideValue(IServiceProvider serviceProvider)
    {
        var localizationService = App.Services?.GetService<ILocalizationService>();
        if (localizationService == null) return Key;

        var binding = new Binding
        {
            Source = localizationService,
            Path = $"[{Key}]",
            Mode = BindingMode.OneWay
        };

        return binding;
    }
}