using CommunityToolkit.Mvvm.ComponentModel;
using AutoFlow.Application.Interfaces;
using Microsoft.Extensions.DependencyInjection;

namespace AutoFlow.App.ViewModels;

public abstract partial class ViewModelBase : ObservableObject
{
    public ILocalizationService L => App.Services?.GetRequiredService<ILocalizationService>()!;
}
