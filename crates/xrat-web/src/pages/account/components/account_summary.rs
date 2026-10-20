use super::*;

#[component]
pub(crate) fn AccountSummary(user: User) -> Element {
    let role = match user.role {
        xrat_web::types::Role::Admin => "Administrator",
        xrat_web::types::Role::Operator => "Operator",
        xrat_web::types::Role::Viewer => "Viewer",
    };
    rsx! {
        section {
            class: concat!(
                "self-account-summary flex items-center pb-6.5 mb-7 border-b border-solid ",
                "[&&_[data-slot=avatar]]:[width:46px] [&&_[data-slot=avatar]]:[height:46px] ",
                "[&&_[data-slot=avatar-fallback]_svg]:[width:22px] [&&_[data-slot=avatar-fallback]_svg]:[height:22px] ",
                "border-b-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] gap-3.5 narrow:flex-wrap ",
                "[&_h2]:text-[18px] [&_h2]:font-semibold [&_h2]:wrap-anywhere [&_h2]:m-0 [&_p]:mt-1 [&_p]:mb-0 ",
                "[&_p]:text-[12px] [&_p]:text-muted-foreground [&_p]:mx-0 ",
            ),
            aria_label: "Current account",
            Avatar { size: AvatarSize::Lg,
                AvatarFallback { UserRound {} }
            }
            div {
                h2 { "{user.username}" }
                p { "{role}" }
            }
            div { class: "ml-auto flex flex-col items-end gap-3 narrow:basis-full narrow:ml-15 narrow:items-start",
                span { class: "self-account-source inline-flex items-center text-[11px] text-muted-foreground gap-1.5 [&_svg]:w-[13px] [&_svg]:h-[13px]",
                    if user.configured {
                        Lock {}
                        "Managed on host"
                    } else {
                        Shield {}
                        "Local account"
                    }
                }
                SignOutButton {}
            }
        }
    }
}
