using DataBase.Entity;
using Riok.Mapperly.Abstractions;

namespace DtoMappers.Mappers;

[Mapper(EnumMappingStrategy = EnumMappingStrategy.ByName)]
public partial class GmapsUserMapper
{
    public static partial GmapsUserDto GmapsUserToDto(GmapsUser gmapsUser);
    public static partial GmapsUser GmapsUserDtoToEntity(GmapsUserDto gmapsUserDto);
}
