use uuid::Uuid;
use uuid_macros::uuid_u128;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u128)]
#[non_exhaustive]
pub enum GptPartitionType {
    MicrosoftBasicData = uuid_u128! {"EBD0A0A2-B9E5-4433-87C0-68B6B72699C7"},
    MicrosoftReserved = uuid_u128! {"E3C9E316-0B5C-4DB8-817D-F92DF00215AE"},
    WindowsRe = uuid_u128! {"DE94BBA4-06D1-4D40-A16A-BFD50179D6AC"},
    ONIEBoot = uuid_u128! {"7412F7D5-A156-4B13-81DC-867174929325"},
    ONIEConfig = uuid_u128! {"D4E6E2CD-4469-46F3-B5CB-1BFF57AFC149"},
    Plan9 = uuid_u128! {"C91818F9-8025-47AF-89D2-F030D7000C2C"},
    PowerPCPrepBoot = uuid_u128! {"9E1A2D38-C612-4316-AA26-8B49521E5A8B"},
    WindowsLDMData = uuid_u128! {"AF9B60A0-1431-4F62-BC68-3311714A69AD"},
    WindowsLDMMetadata = uuid_u128! {"5808C8AA-7E8F-42E0-85D2-E1E90434CFB3"},
    WindowsStorageSpaces = uuid_u128! {"E75CAF8F-F680-4CEE-AFA3-B001E56EFC2D"},
    IBMGPFS = uuid_u128! {"37AFFC90-EF7D-4E96-91C3-2D7AE055B174"},
    ChromeOSKernel = uuid_u128! {"FE3A2A5D-4F32-41A7-B725-ACCC3285A309"},
    ChromeOSRoot = uuid_u128! {"3CB8E202-3B7E-47DD-8A3C-7FF2A13CFCEC"},
    ChromeOSReserved = uuid_u128! {"2E0A753D-9E48-43B0-8337-B15192CB1B5E"},
    LinuxSwap = uuid_u128! {"0657FD6D-A4AB-43C4-84E5-0933C84B4F4F"},
    LinuxFilesystem = uuid_u128! {"0FC63DAF-8483-4772-8E79-3D69D8477DE4"},
    LinuxReserved = uuid_u128! {"8DA63339-0007-60C0-C436-083AC8230908"},
    LinuxHome = uuid_u128! {"933AC7E1-2EB4-4F13-B844-0E14E2AEF915"},
    LinuxX86Root = uuid_u128! {"44479540-F297-41B2-9AF7-D131D5F0458A"},
    LinuxX64Root = uuid_u128! {"4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709"},
    LinuxARM64Root = uuid_u128! {"B921B045-1DF0-41C3-AF44-4C6F280D3FAE"},
    LinuxSrv = uuid_u128! {"3B8F8425-20E0-4F3B-907F-1A25A76F98E8"},
    LinuxARM32Root = uuid_u128! {"69DAD710-2CE4-4E3C-B16C-21A1D49ABED3"},
    LinuxDMCrypt = uuid_u128! {"7FFEC5C9-2D00-49B7-8941-3EA10A5586B7"},
    LinuxIA64Root = uuid_u128! {"CA7D7CCB-63ED-4C53-861C-1742536059CC"},
    LinuxX86RootVerity = uuid_u128! {"993D8D3D-F80E-4225-855A-9DAF8ED7EA97"},
    LinuxX64RootVerity = uuid_u128! {"2C7357ED-EBD2-46D9-AEC1-23D437EC2BF5"},
    LinuxARM32RootVerity = uuid_u128! {"7386CDF2-203C-47A9-A498-F2ECCE45A2D6"},
    LinuxARM64RootVerity = uuid_u128! {"DF3300CE-D69F-4C92-978C-9BFB0F38D820"},
    LinuxIA64RootVerity = uuid_u128! {"86ED10D5-B607-45BB-8957-D350F23D0571"},
    LinuxVar = uuid_u128! {"4D21B016-B534-45C2-A9FB-5C16E091FD2D"},
    LinuxVarTmp = uuid_u128! {"7EC6F557-3BC5-4ACA-B293-16EF5DF639D1"},
    IntelRapidStart = uuid_u128! {"D3BFE2DE-3DAF-11DF-BA40-E3A556D89593"},
    ContainerLinuxUsr = uuid_u128! {"5DFBF5F4-2848-4BAC-AA5E-0D9A20B745A6"},
    ContainerLinuxRoot = uuid_u128! {"3884DD41-8582-4404-B9A8-E9B84F2DF50E"},
    ContainerLinuxOem = uuid_u128! {"C95DC21A-DF0E-4340-8D7B-26CBFA9A03E0"},
    ContainerLinuxRootOnRaid = uuid_u128! {"BE9067B9-EA49-4F15-B4F6-F36F8C9E1818"},
    LinuxLVM = uuid_u128! {"E6D6D379-F507-44C2-A23C-238F2A3DF928"},
    AndroidBootloader = uuid_u128! {"2568845D-2332-4675-BC39-8FA5A4748D15"},
    AndroidBootloader2 = uuid_u128! {"114EAFFE-1552-4022-B26E-9B053604CF84"},
    AndroidBoot1 = uuid_u128! {"49A4D17F-93A3-45C1-A0DE-F50B2EBE2599"},
    AndroidRecovery1 = uuid_u128! {"4177C722-9E92-4AAB-8644-43502BFD5506"},
    AndroidMisc = uuid_u128! {"EF32A33B-A409-486C-9141-9FFB711F6266"},
    AndroidMetadata = uuid_u128! {"20AC26BE-20B7-11E3-84C5-6CFDB94711E9"},
    AndroidSystem = uuid_u128! {"38F428E6-D326-425D-9140-6E0EA133647C"},
    AndroidCache = uuid_u128! {"A893EF21-E428-470A-9E55-0668FD91A2D9"},
    AndroidData = uuid_u128! {"DC76DDA9-5AC1-491C-AF42-A82591580C0D"},
    AndroidPersistent = uuid_u128! {"EBC597D0-2053-4B15-8B64-E0AAC75F4DB1"},
    AndroidFactory = uuid_u128! {"8F68CC74-C5E5-48DA-BE91-A0C8C15E9C80"},
    AndroidFastbootTertiary = uuid_u128! {"767941D0-2085-11E3-AD3B-6CFDB94711E9"},
    AndroidOem = uuid_u128! {"AC6D7924-EB71-4DF8-B48D-E267B27148FF"},
    AndroidVendor = uuid_u128! {"C5A0AEEC-13EA-11E5-A1B1-001E67CA0C3C"},
    AndroidConfig = uuid_u128! {"BD59408B-4514-490D-BF12-9878D963F378"},
    AndroidFactoryAlt = uuid_u128! {"9FDAA6EF-4B3F-40D2-BA8D-BFF16BFB887B"},
    AndroidMeta = uuid_u128! {"19A710A2-B3CA-11E4-B026-10604B889DCF"},
    AndroidExt = uuid_u128! {"193D1EA4-B3CA-11E4-B075-10604B889DCF"},
    AndroidSbl1 = uuid_u128! {"DEA0BA2C-CBDD-4805-B4F9-F428251C3E98"},
    AndroidSbl2 = uuid_u128! {"8C6B52AD-8A9E-4398-AD09-AE916E53AE2D"},
    AndroidSbl3 = uuid_u128! {"05E044DF-92F1-4325-B69E-374A82E97D6E"},
    AndroidAppSbl = uuid_u128! {"400FFDCD-22E0-47E7-9A23-F16ED9382388"},
    AndroidQseeTz = uuid_u128! {"A053AA7F-40B8-4B1C-BA08-2F68AC71A4F4"},
    AndroidQheeHyp = uuid_u128! {"E1A6A689-0C8D-4CC6-B4E8-55A4320FBD8A"},
    AndroidRpm = uuid_u128! {"098DF793-D712-413D-9D4E-89D711772228"},
    AndroidWdogDebugSdi = uuid_u128! {"D4E0D938-B7FA-48C1-9D21-BC5ED5C4B203"},
    AndroidDdr = uuid_u128! {"20A0C19C-286A-42FA-9CE7-F64C3226A794"},
    AndroidCdt = uuid_u128! {"A19F205F-CCD8-4B6D-8F1E-2D9BC24CFFB1"},
    AndroidRamDump = uuid_u128! {"66C9B323-F7FC-48B6-BF96-6F32E335A428"},
    AndroidSec = uuid_u128! {"303E6AC3-AF15-4C54-9E9B-D9A8FBECF401"},
    AndroidPmic = uuid_u128! {"C00EEF24-7709-43D6-9799-DD2B411E7A3C"},
    AndroidMisc1 = uuid_u128! {"82ACC91F-357C-4A68-9C8F-689E1B1A23A1"},
    AndroidMisc2 = uuid_u128! {"E2802D54-0545-E8A1-A1E8-C7A3E245ACD4"},
    AndroidDeviceInfo = uuid_u128! {"65ADDCF4-0C5C-4D9A-AC2D-D90B5CBFCD03"},
    AndroidApdp = uuid_u128! {"E6E98DA2-E22A-4D12-AB33-169E7DEAA507"},
    AndroidMsadp = uuid_u128! {"ED9E8101-05FA-46B7-82AA-8D58770D200B"},
    AndroidDpo = uuid_u128! {"11406F35-1173-4869-807B-27DF71802812"},
    AndroidRecovery2 = uuid_u128! { "9D72D4E4-9958-42DA-AC26-BEA7A90B0434"},
    AndroidPersist = uuid_u128! {"6C95E238-E343-4BA8-B489-8681ED22AD0B"},
    AndroidSt1 = uuid_u128! {"EBBEADAF-22C9-E33B-8F5D-0E81686A68CB"},
    AndroidSt2 = uuid_u128! {"0A288B1F-22C9-E33B-8F5D-0E81686A68CB"},
    AndroidFsc = uuid_u128! {"57B90A16-22C9-E33B-8F5D-0E81686A68CB"},
    AndroidFsg1 = uuid_u128! { "638FF8E2-22C9-E33B-8F5D-0E81686A68CB"},
    AndroidFsg2 = uuid_u128! {"2013373E-1AC4-4131-BFD8-B6A7AC638772"},
    AndroidSsd = uuid_u128! {"2C86E742-745E-4FDD-BFD8-B6A7AC638772"},
    AndroidKeystore = uuid_u128! {"DE7D4029-0F5B-41C8-AE7E-F6C023A02B33"},
    AndroidEncrypt = uuid_u128! {"323EF595-AF7A-4AFA-8060-97BE72841BB9"},
    AndroidEksst = uuid_u128! {"45864011-CF89-46E6-A445-85262E065604"},
    AndroidRct = uuid_u128! {"8ED8AE95-597F-4C8A-A5BD-A7FF8E4DFAA9"},
    AndroidSpare1 = uuid_u128! {"DF24E5ED-8C96-4B86-B00B-79667DC6DE11"},
    AndroidSpare2 = uuid_u128! {"7C29D3AD-78B9-452E-9DEB-D098D542F092"},
    AndroidSpare3 = uuid_u128! { "379D107E-229E-499D-AD4F-61F5BCF87BD4"},
    AndroidSpare4 = uuid_u128! {"0DEA65E5-A676-4CDF-823C-77568B577ED5"},
    AndroidRawResources = uuid_u128! {"4627AE27-CFEF-48A1-88FE-99C3509ADE26"},
    AndroidBoot2 = uuid_u128! {"20117F86-E985-4357-B9EE-374BC1D8487D"},
    AndroidFota = uuid_u128! {"86A7CB80-84E1-408C-99AB-694F1A410FC7"},
    AndroidSystem2 = uuid_u128! {"97D7B011-54DA-4835-B3C4-917AD6E73D74"},
    AndroidCache2 = uuid_u128! {"5594C694-C871-4B5F-90B1-690A6F68E0F7"},
    AndroidUserData = uuid_u128! {"1B81E7E6-F50D-419B-A739-2AEEF8DA3335"},
    AndroidLaf = uuid_u128! {"98523EC6-90FE-4C67-B50A-0FC59ED6F56D"},
    AndroidPG1FS = uuid_u128! {"2644BCC0-F36A-4792-9533-1738BED53EE3"},
    AndroidPG2FS = uuid_u128! {"DD7C91E9-38C9-45C5-8A12-4A80F7E14057"},
    AndroidBoardInfo = uuid_u128! { "7696D5B6-43FD-4664-A228-C563C4A1E8CC"},
    AndroidMfg = uuid_u128! {"0D802D54-058D-4A20-AD2D-C7A362CEACD4"},
    AndroidLimits = uuid_u128! {"10A0C19C-516A-5444-5CE3-664C3226A794"},
    FreeBSDDiskLabel = uuid_u128! {"516E7CB4-6ECF-11D6-8FF8-00022D09712B"},
    FreeBSDBoot = uuid_u128! {"83BD6B9D-7F41-11DC-BE0B-001560B84F0F"},
    FreeBSDSwap = uuid_u128! {"516E7CB5-6ECF-11D6-8FF8-00022D09712B"},
    FreeBSDUfs = uuid_u128! {"516E7CB6-6ECF-11D6-8FF8-00022D09712B"},
    FreeBSDZfs = uuid_u128! {"516E7CBA-6ECF-11D6-8FF8-00022D09712B"},
    FreeBSDVinumRaid = uuid_u128! {"516E7CB8-6ECF-11D6-8FF8-00022D09712B"},
    MidnightBSDData = uuid_u128! {"85D5E45A-237C-11E1-B4B3-E89A8F7FC3A7"},
    MidnightBSDBoot = uuid_u128! {"85D5E45E-237C-11E1-B4B3-E89A8F7FC3A7"},
    MidnightBSDSwap = uuid_u128! {"85D5E45B-237C-11E1-B4B3-E89A8F7FC3A7"},
    MidnightBSDUfs = uuid_u128! {"0394Ef8B-237E-11E1-B4B3-E89A8F7FC3A7"},
    MidnightBSDZfs = uuid_u128! {"85D5E45D-237C-11E1-B4B3-E89A8F7FC3A7"},
    MidnightBSDVinum = uuid_u128! {"85D5E45C-237C-11E1-B4B3-E89A8F7FC3A7"},
    OpenBSDDiskLabel = uuid_u128! {"824CC7A0-36A8-11E3-890A-952519AD3F61"},
    AppleUfs = uuid_u128! {"55465300-0000-11AA-AA11-00306543ECAC"},
    NetBSDSwap = uuid_u128! {"49F48D32-B10E-11DC-B99B-0019D1879648"},
    NetBSDFfs = uuid_u128! {"49F48D5A-B10E-11DC-B99B-0019D1879648"},
    NetBSDLfs = uuid_u128! {"49F48D82-B10E-11DC-B99B-0019D1879648"},
    NetBSDConcatenated = uuid_u128! {"2DB519C4-B10F-11DC-B99B-0019D1879648"},
    NetBSDEncrypted = uuid_u128! {"2DB519EC-B10F-11DC-B99B-0019D1879648"},
    NetBSDRaid = uuid_u128! {"49F48DAA-B10E-11DC-B99B-0019D1879648"},
    AppleRecoveryHd = uuid_u128! {"426F6F74-0000-11AA-AA11-00306543ECAC"},
    AppleHfs = uuid_u128! {"48465300-0000-11AA-AA11-00306543ECAC"},
    AppleRaid = uuid_u128! {"52414944-0000-11AA-AA11-00306543ECAC"},
    AppleRaidOffline = uuid_u128! {"52414944-5F4F-11AA-AA11-00306543ECAC"},
    AppleLabel = uuid_u128! {"4C616265-6C00-11AA-AA11-00306543ECAC"},
    AppleTvRecovery = uuid_u128! {"5265636F-7665-11AA-AA11-00306543ECAC"},
    AppleCoreStorage = uuid_u128! {"53746F72-6167-11AA-AA11-00306543ECAC"},
    AppleSoftRaidStatus = uuid_u128! {"B6FA30DA-92D2-4A9A-96F1-871EC6486200"},
    AppleSoftRaidScratch = uuid_u128! {"2E313465-19B9-463F-8126-8A7993773801"},
    AppleSoftRaidVolume = uuid_u128! {"FA709C7E-65B1-4593-BFD5-E71D61DE9B02"},
    AppleSoftRaidCache = uuid_u128! {"BBBA6DF5-F46F-4A89-8F59-8765B2727503"},
    AppleApfs = uuid_u128! {"7C3457EF-0000-11AA-AA11-00306543ECAC"},
    Qnx6PowerSafe = uuid_u128! {"CEF5A9AD-73BC-4601-89F3-CDEEEEE321A1"},
    AcronisSecureZone = uuid_u128! {"0311FC50-01CA-4725-AD77-9ADBB20ACE98"},
    SolarisBoot = uuid_u128! {"6A82CB45-1DD2-11B2-99A6-080020736631"},
    SolarisRoot = uuid_u128! {"6A85CF4D-1DD2-11B2-99A6-080020736631"},
    SolarisUsrOrMacZfs = uuid_u128! {"6A898CC3-1DD2-11B2-99A6-080020736631"},
    SolarisSwap = uuid_u128! {"6A87C46F-1DD2-11B2-99A6-080020736631"},
    SolarisBackup = uuid_u128! {"6A8B642B-1DD2-11B2-99A6-080020736631"},
    SolarisVar = uuid_u128! {"6A8EF2E9-1DD2-11B2-99A6-080020736631"},
    SolarisHome = uuid_u128! {"6A90BA39-1DD2-11B2-99A6-080020736631"},
    SolarisAlternateSector = uuid_u128! {"6A9283A5-1DD2-11B2-99A6-080020736631"},
    SolarisReserved1 = uuid_u128! {"6A945A3B-1DD2-11B2-99A6-080020736631"},
    SolarisReserved2 = uuid_u128! {"6A9630D1-1DD2-11B2-99A6-080020736631"},
    SolarisReserved3 = uuid_u128! {"6A980767-1DD2-11B2-99A6-080020736631"},
    SolarisReserved4 = uuid_u128! {"6A96237F-1DD2-11B2-99A6-080020736631"},
    SolarisReserved5 = uuid_u128! {"6A8D2AC7-1DD2-11B2-99A6-080020736631"},
    HpUxData = uuid_u128! {"75894C1E-3AEB-11D3-B7C1-7B03A0000000"},
    HpUxService = uuid_u128! {"E2A1E728-32E3-11D6-A682-7B03A0000000"},
    VeracryptData = uuid_u128! {"8C8F8EFF-AC95-4770-814A-21994F2DBC8F"},
    FreeDesktopBoot = uuid_u128! {"BC13C2FF-59E6-4262-A352-B275FD6F7172"},
    HaikuBfs = uuid_u128! {"42465331-3BA3-10F1-802A-4861696B7521"},
    SonySystemPartition = uuid_u128! {"F4019732-066E-4E12-8273-346C5641494F"},
    LenovoSystemPartition = uuid_u128! {"BFBFAFE7-A34F-448A-9A5B-6213EB736C22"},
    EFISystemPartition = uuid_u128! {"C12A7328-F81F-11D2-BA4B-00A0C93EC93B"},
    MBRPartitionScheme = uuid_u128! {"024DEE41-33E7-11D3-9D69-0008C781F39F"},
    BIOSBootPartition = uuid_u128! {"21686148-6449-6E6F-744E-656564454649"},
    CephOSD = uuid_u128! {"4FBD7E29-9D25-41B8-AFD0-062C0CEFF05D"},
    CephDmCryptOSD = uuid_u128! {"4FBD7E29-9D25-41B8-AFD0-5EC00CEFF05D"},
    CephJournal = uuid_u128! {"45B0969E-9B03-4F30-B4C6-B4B80CEFF106"},
    CephDmCryptJournal = uuid_u128! {"45B0969E-9B03-4F30-B4C6-5EC00CEFF106"},
    CephDiskInCreation = uuid_u128! {"89C57F98-2FE5-4DC0-89C1-F3AD0CEFF2BE"},
    CephDmCryptDiskInCreation = uuid_u128! {"89C57F98-2FE5-4DC0-89C1-5EC00CEFF2BE"},
    CephBlock = uuid_u128! {"CAFECAFE-9B03-4F30-B4C6-B4B80CEFF106"},
    CephBlockDB = uuid_u128! {"30CD0809-C2B2-499C-8879-2D6B78529876"},
    CephBlockWriteAheadLog = uuid_u128! {"5CE17FCE-4087-4169-B7FF-056CC58473F9"},
    CephLockBoxForDmCryptKeys = uuid_u128! {"FB3AABF9-D25F-47CC-BF5E-721D1816496B"},
    CephMultiPathOSD = uuid_u128! {"4FBD7E29-8AE0-4982-BF9D-5A8D867AF560"},
    CephMultiPathJournal = uuid_u128! {"45B0969E-8AE0-4982-BF9D-5A8D867AF560"},
    CephMultipathBlock1 = uuid_u128! {"CAFECAFE-8AE0-4982-BF9D-5A8D867AF560"},
    CephMultipathBlock2 = uuid_u128! {"7F4A666A-16F3-47A2-8445-152EF4D03F6C"},
    CephMultipathBlockDB = uuid_u128! {"EC6D6385-E346-45DC-BE91-DA2A7C8B3261"},
    CephMultipathBlockWriteAheadLog = uuid_u128! {"01B41E1B-002A-453C-9F17-88793989FF8F"},
    CephDmCryptBlock = uuid_u128! {"CAFECAFE-9B03-4F30-B4C6-5EC00CEFF106"},
    CephDmCryptBlockDB = uuid_u128! {"93B0052D-02D9-4D8A-A43B-33A3EE4DFBC3"},
    CephDmCryptBlockWriteAheadLog = uuid_u128! {"306E8683-4FE2-4330-B7C0-00A917C16966"},
    CephDmCryptBlockLuksJournal = uuid_u128! {"45B0969E-9B03-4F30-B4C6-35865CEFF106"},
    CephDmCryptLuksBlock = uuid_u128! {"CAFECAFE-9B03-4F30-B4C6-35865CEFF106"},
    CephDmCryptLuksBlockDB = uuid_u128! {"166418DA-C469-4022-ADF4-B30AFD37F176"},
    CephDmCryptLuksBlockWriteAheadLog = uuid_u128! {"86A32090-3647-40B9-BBBD-38D8C573AA86"},
    CephDmCryptLuksOSD = uuid_u128! {"4FBD7E29-9D25-41B8-AFD0-35865CEFF05D"},
    VMWareVMFS = uuid_u128! {"AA31E02A-400F-11DB-9590-000C2911D1B8"},
    VMWareReserved = uuid_u128! {"9198EFFC-31C0-11DB-8F78-000C2911D1B8"},
    VMWareKCoreCrashProtection = uuid_u128! {"9D275380-40AD-11DB-BF97-000C2911D1B8"},
    LinuxRAID = uuid_u128! {"A19D880F-05FC-4D3B-A006-743F0F84911E"},
}

impl GptPartitionType {
    pub fn to_guid(&self) -> Uuid {
        Uuid::from_u128(*self as u128)
    }

    pub fn as_str(&self) -> &'static str {
        uuid128_partition_type_guid_to_name(*self).unwrap()
    }
}

// Rust currently does not support generating static maps
// at compile time
pub fn uuid128_partition_type_guid_to_name(t: GptPartitionType) -> Option<&'static str> {
    match t {
        GptPartitionType::MicrosoftBasicData => Some("Microsoft basic data"),
        GptPartitionType::MicrosoftReserved => Some("Microsoft reserved"),
        GptPartitionType::WindowsRe => Some("Windows RE"),
        GptPartitionType::ONIEBoot => Some("ONIE Boot"),
        GptPartitionType::ONIEConfig => Some("ONIE Config"),
        GptPartitionType::Plan9 => Some("Plan 9"),
        GptPartitionType::PowerPCPrepBoot => Some("PowerPC PReP Boot"),
        GptPartitionType::WindowsLDMData => Some("Windows LDM data"),
        GptPartitionType::WindowsLDMMetadata => Some("Windows LDM metadata"),
        GptPartitionType::WindowsStorageSpaces => Some("Windows Storage Spaces"),
        GptPartitionType::IBMGPFS => Some("IBM GPFS"),
        GptPartitionType::ChromeOSKernel => Some("ChromeOS kernel"),
        GptPartitionType::ChromeOSRoot => Some("ChromeOS root"),
        GptPartitionType::ChromeOSReserved => Some("ChromeOS reserved"),
        GptPartitionType::LinuxSwap => Some("Linux swap"),
        GptPartitionType::LinuxFilesystem => Some("Linux filesystem"),
        GptPartitionType::LinuxReserved => Some("Linux reserved"),
        GptPartitionType::LinuxHome => Some("Linux /home"),
        GptPartitionType::LinuxX86Root => Some("Linux x86 root"),
        GptPartitionType::LinuxX64Root => Some("Linux x86-64 root"),
        GptPartitionType::LinuxARM64Root => Some("Linux ARM64 root"),
        GptPartitionType::LinuxSrv => Some("Linux /srv"),
        GptPartitionType::LinuxARM32Root => Some("Linux ARM32 root"),
        GptPartitionType::LinuxDMCrypt => Some("Linux dm-crypt"),
        GptPartitionType::LinuxIA64Root => Some("Linux IA64 root"),
        GptPartitionType::LinuxX86RootVerity => Some("Linux x86 root verity"),
        GptPartitionType::LinuxX64RootVerity => Some("Linux x64 root verity"),
        GptPartitionType::LinuxARM32RootVerity => Some("Linux ARM32 root verity"),
        GptPartitionType::LinuxARM64RootVerity => Some("Linux ARM64 root verity"),
        GptPartitionType::LinuxIA64RootVerity => Some("Linux IA64 root verity"),
        GptPartitionType::LinuxVar => Some("Linux /var"),
        GptPartitionType::LinuxVarTmp => Some("Linux /var/tmp"),
        GptPartitionType::IntelRapidStart => Some("Intel Rapid Start"),
        GptPartitionType::ContainerLinuxUsr => Some("Container Linux /usr"),
        GptPartitionType::ContainerLinuxRoot => Some("Container Linux resizable rootfs"),
        GptPartitionType::ContainerLinuxOem => Some("Container Linux OEM"),
        GptPartitionType::ContainerLinuxRootOnRaid => Some("Container Linux root on RAID"),
        GptPartitionType::LinuxLVM => Some("Linux LVM"),
        GptPartitionType::AndroidBootloader => Some("Android bootloader"),
        GptPartitionType::AndroidBootloader2 => Some("Android bootloader 2"),
        GptPartitionType::AndroidBoot1 => Some("Android boot 1"),
        GptPartitionType::AndroidRecovery1 => Some("Android recovery 1"),
        GptPartitionType::AndroidMisc => Some("Android misc"),
        GptPartitionType::AndroidMetadata => Some("Android metadata"),
        GptPartitionType::AndroidSystem => Some("Android system"),
        GptPartitionType::AndroidCache => Some("Android cache"),
        GptPartitionType::AndroidData => Some("Android data"),
        GptPartitionType::AndroidPersistent => Some("Android persistent"),
        GptPartitionType::AndroidFactory => Some("Android factory"),
        GptPartitionType::AndroidFastbootTertiary => Some("Android fastboot/tertiary"),
        GptPartitionType::AndroidOem => Some("Android OEM"),
        GptPartitionType::AndroidVendor => Some("Android vendor"),
        GptPartitionType::AndroidConfig => Some("Android config"),
        GptPartitionType::AndroidFactoryAlt => Some("Android factory (alt)"),
        GptPartitionType::AndroidMeta => Some("Android meta"),
        GptPartitionType::AndroidExt => Some("Android EXT"),
        GptPartitionType::AndroidSbl1 => Some("Android SBL1"),
        GptPartitionType::AndroidSbl2 => Some("Android SBL2"),
        GptPartitionType::AndroidSbl3 => Some("Android SBL3"),
        GptPartitionType::AndroidAppSbl => Some("Android APPSBL"),
        GptPartitionType::AndroidQseeTz => Some("Android QSSE/tz"),
        GptPartitionType::AndroidQheeHyp => Some("Android QHEE/hyp"),
        GptPartitionType::AndroidRpm => Some("Android RPM"),
        GptPartitionType::AndroidWdogDebugSdi => Some("Android WDOG debug/sdi"),
        GptPartitionType::AndroidDdr => Some("Android DDR"),
        GptPartitionType::AndroidCdt => Some("Android CDT"),
        GptPartitionType::AndroidRamDump => Some("Android RAM dump"),
        GptPartitionType::AndroidSec => Some("Android SEC"),
        GptPartitionType::AndroidPmic => Some("Android PMIC"),
        GptPartitionType::AndroidMisc1 => Some("Android misc 1"),
        GptPartitionType::AndroidMisc2 => Some("Android misc 2"),
        GptPartitionType::AndroidDeviceInfo => Some("Android device info"),
        GptPartitionType::AndroidApdp => Some("Android APDP"),
        GptPartitionType::AndroidMsadp => Some("Android MSADP"),
        GptPartitionType::AndroidDpo => Some("Android DPO"),
        GptPartitionType::AndroidRecovery2 => Some("Android recovery 2"),
        GptPartitionType::AndroidPersist => Some("Android persist"),
        GptPartitionType::AndroidSt1 => Some("Android modem ST1"),
        GptPartitionType::AndroidSt2 => Some("Android modem ST2"),
        GptPartitionType::AndroidFsc => Some("Android FSC"),
        GptPartitionType::AndroidFsg1 => Some("Android FSG1"),
        GptPartitionType::AndroidFsg2 => Some("Android FSG2"),
        GptPartitionType::AndroidSsd => Some("Android SSD"),
        GptPartitionType::AndroidKeystore => Some("Android keystore"),
        GptPartitionType::AndroidEncrypt => Some("Android encrypt"),
        GptPartitionType::AndroidEksst => Some("Android EKSST"),
        GptPartitionType::AndroidRct => Some("Android RCT"),
        GptPartitionType::AndroidSpare1 => Some("Android spare1"),
        GptPartitionType::AndroidSpare2 => Some("Android spare2"),
        GptPartitionType::AndroidSpare3 => Some("Android spare3"),
        GptPartitionType::AndroidSpare4 => Some("Android spare4"),
        GptPartitionType::AndroidRawResources => Some("Android raw resources"),
        GptPartitionType::AndroidBoot2 => Some("Android boot 2"),
        GptPartitionType::AndroidFota => Some("Android FOTA"),
        GptPartitionType::AndroidSystem2 => Some("Android system 2"),
        GptPartitionType::AndroidCache2 => Some("Android cache 2"),
        GptPartitionType::AndroidUserData => Some("Android userdata"),
        GptPartitionType::AndroidLaf => Some("LG (Android) advanced flasher"),
        GptPartitionType::AndroidPG1FS => Some("Android PG1FS"),
        GptPartitionType::AndroidPG2FS => Some("Android PG2FS"),
        GptPartitionType::AndroidBoardInfo => Some("Android board info"),
        GptPartitionType::AndroidMfg => Some("Android MFG"),
        GptPartitionType::AndroidLimits => Some("Android limits"),
        GptPartitionType::FreeBSDDiskLabel => Some("FreeBSD disklabel"),
        GptPartitionType::FreeBSDBoot => Some("FreeBSD boot"),
        GptPartitionType::FreeBSDSwap => Some("FreeBSD swap"),
        GptPartitionType::FreeBSDUfs => Some("FreeBSD UFS"),
        GptPartitionType::FreeBSDZfs => Some("FreeBSD ZFS"),
        GptPartitionType::FreeBSDVinumRaid => Some("FreeBSD Vinum/RAID"),
        GptPartitionType::MidnightBSDData => Some("Midnight BSD data"),
        GptPartitionType::MidnightBSDBoot => Some("Midnight BSD boot"),
        GptPartitionType::MidnightBSDSwap => Some("Midnight BSD swap"),
        GptPartitionType::MidnightBSDUfs => Some("Midnight BSD UFS"),
        GptPartitionType::MidnightBSDZfs => Some("Midnight BSD ZFS"),
        GptPartitionType::MidnightBSDVinum => Some("Midnight BSD Vinum"),
        GptPartitionType::OpenBSDDiskLabel => Some("OpenBSD disklabel"),
        GptPartitionType::AppleUfs => Some("Apple UFS"),
        GptPartitionType::NetBSDSwap => Some("NetBSD swap"),
        GptPartitionType::NetBSDFfs => Some("NetBSD FFS"),
        GptPartitionType::NetBSDLfs => Some("NetBSD LFS"),
        GptPartitionType::NetBSDConcatenated => Some("NetBSD concatenated"),
        GptPartitionType::NetBSDEncrypted => Some("NetBSD encrypted"),
        GptPartitionType::NetBSDRaid => Some("NetBSD RAID"),
        GptPartitionType::AppleRecoveryHd => Some("Apple Recovery HD"),
        GptPartitionType::AppleHfs => Some("Apple HFS"),
        GptPartitionType::AppleRaid => Some("Apple RAID"),
        GptPartitionType::AppleRaidOffline => Some("Apple RAID offline"),
        GptPartitionType::AppleLabel => Some("Apple label"),
        GptPartitionType::AppleTvRecovery => Some("Apple TV Recovery"),
        GptPartitionType::AppleCoreStorage => Some("Apple Core Storage"),
        GptPartitionType::AppleSoftRaidStatus => Some("Apple SoftRAID Status"),
        GptPartitionType::AppleSoftRaidScratch => Some("Apple SoftRAID Scratch"),
        GptPartitionType::AppleSoftRaidVolume => Some("Apple SoftRAID Volume"),
        GptPartitionType::AppleSoftRaidCache => Some("Apple SoftRAID Cache"),
        GptPartitionType::AppleApfs => Some("Apple APFS"),
        GptPartitionType::Qnx6PowerSafe => Some("QNX6 Power-Safe"),
        GptPartitionType::AcronisSecureZone => Some("Acronis Secure Zone"),
        GptPartitionType::SolarisBoot => Some("Solaris boot"),
        GptPartitionType::SolarisRoot => Some("Solaris root"),
        GptPartitionType::SolarisUsrOrMacZfs => Some("Solaris /usr & Mac ZFS"),
        GptPartitionType::SolarisSwap => Some("Solaris swap"),
        GptPartitionType::SolarisBackup => Some("Solaris backup"),
        GptPartitionType::SolarisVar => Some("Solaris /var"),
        GptPartitionType::SolarisHome => Some("Solaris /home"),
        GptPartitionType::SolarisAlternateSector => Some("Solaris alternate sector"),
        GptPartitionType::SolarisReserved1 => Some("Solaris Reserved 1"),
        GptPartitionType::SolarisReserved2 => Some("Solaris Reserved "),
        GptPartitionType::SolarisReserved3 => Some("Solaris Reserved "),
        GptPartitionType::SolarisReserved4 => Some("Solaris Reserved "),
        GptPartitionType::SolarisReserved5 => Some("Solaris Reserved "),
        GptPartitionType::HpUxData => Some("HP-UX data"),
        GptPartitionType::HpUxService => Some("HP-UX service"),
        GptPartitionType::VeracryptData => Some("Veracrypt Data"),
        GptPartitionType::FreeDesktopBoot => Some("Freedesktop $BOOT"),
        GptPartitionType::HaikuBfs => Some("Haiku BFS"),
        GptPartitionType::SonySystemPartition => Some("Sony system partition"),
        GptPartitionType::LenovoSystemPartition => Some("Lenovo system partition"),
        GptPartitionType::EFISystemPartition => Some("EFI system partition"),
        GptPartitionType::MBRPartitionScheme => Some("MBR partition scheme"),
        GptPartitionType::BIOSBootPartition => Some("BIOS boot partition"),
        GptPartitionType::CephOSD => Some("Ceph OSD"),
        GptPartitionType::CephDmCryptOSD => Some("Ceph dm-crypt OSD"),
        GptPartitionType::CephJournal => Some("Ceph journal"),
        GptPartitionType::CephDmCryptJournal => Some("Ceph dm-crypt journal"),
        GptPartitionType::CephDiskInCreation => Some("Ceph disk in creation"),
        GptPartitionType::CephDmCryptDiskInCreation => Some("Ceph dm-crypt disk in creation"),
        GptPartitionType::CephBlock => Some("Ceph block"),
        GptPartitionType::CephBlockDB => Some("Ceph block DB"),
        GptPartitionType::CephBlockWriteAheadLog => Some("Ceph block write-ahead log"),
        GptPartitionType::CephLockBoxForDmCryptKeys => Some("Ceph lockbox for dm-crypt keys"),
        GptPartitionType::CephMultiPathOSD => Some("Ceph multipath OSD"),
        GptPartitionType::CephMultiPathJournal => Some("Ceph multipath journal"),
        GptPartitionType::CephMultipathBlock1 => Some("Ceph multipath block 1"),
        GptPartitionType::CephMultipathBlock2 => Some("Ceph multipath block 2"),
        GptPartitionType::CephMultipathBlockDB => Some("Ceph multipath block DB"),
        GptPartitionType::CephMultipathBlockWriteAheadLog => {
            Some("Ceph multipath block write-ahead log")
        }
        GptPartitionType::CephDmCryptBlock => Some("Ceph dm-crypt block"),
        GptPartitionType::CephDmCryptBlockDB => Some("Ceph dm-crypt block DB"),
        GptPartitionType::CephDmCryptBlockWriteAheadLog => {
            Some("Ceph dm-crypt block write-ahead log")
        }
        GptPartitionType::CephDmCryptBlockLuksJournal => Some("Ceph dm-crypt LUKS journal"),
        GptPartitionType::CephDmCryptLuksBlock => Some("Ceph dm-crypt LUKS block"),
        GptPartitionType::CephDmCryptLuksBlockDB => Some("Ceph dm-crypt LUKS block DB"),
        GptPartitionType::CephDmCryptLuksBlockWriteAheadLog => {
            Some("Ceph dm-crypt LUKS block write-ahead log")
        }
        GptPartitionType::CephDmCryptLuksOSD => Some("Ceph dm-crypt LUKS OSD"),
        GptPartitionType::VMWareVMFS => Some("VMWare VMFS"),
        GptPartitionType::VMWareReserved => Some("VMWare reserved"),
        GptPartitionType::VMWareKCoreCrashProtection => Some("VMWare kcore crash protection"),
        GptPartitionType::LinuxRAID => Some("Linux RAID"),
        #[allow(unreachable_patterns)]
        _ => None,
    }
}
